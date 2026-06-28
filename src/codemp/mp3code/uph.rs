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
	
	$Id: uph.c,v 1.3 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/****  uph.c  ***************************************************

Layer 3 audio
 huffman decode


******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_uchar, c_uint};

use super::htable_h::{
    huff_table_1, huff_table_10, huff_table_11, huff_table_12, huff_table_13, huff_table_15,
    huff_table_16, huff_table_2, huff_table_24, huff_table_3, huff_table_5, huff_table_6,
    huff_table_7, huff_table_8, huff_table_9, HUFF_ELEMENT_PTR,
};
use super::l3_h::{BITDAT, HUFF_ELEMENT};

/*===============================================================*/

/* max bits required for any lookup - change if htable changes */
/* quad required 10 bit w/signs  must have (MAXBITS+2) >= 10   */
const MAXBITS: c_int = 9;

static huff_table_0: [HUFF_ELEMENT; 4] = [
    HUFF_ELEMENT_PTR(0),
    HUFF_ELEMENT_PTR(0),
    HUFF_ELEMENT_PTR(0),
    HUFF_ELEMENT_PTR(64),
]; /* dummy must not use */

/*-- 6 bit lookup (purgebits, value) --*/
static quad_table_a: [[c_uchar; 2]; 64] = [
    [6, 11], [6, 15], [6, 13], [6, 14], [6, 7], [6, 5], [5, 9],
    [5, 9], [5, 6], [5, 6], [5, 3], [5, 3], [5, 10], [5, 10],
    [5, 12], [5, 12], [4, 2], [4, 2], [4, 2], [4, 2], [4, 1],
    [4, 1], [4, 1], [4, 1], [4, 4], [4, 4], [4, 4], [4, 4],
    [4, 8], [4, 8], [4, 8], [4, 8], [1, 0], [1, 0], [1, 0],
    [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0],
    [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0],
    [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0],
    [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0], [1, 0],
    [1, 0],
];

#[repr(C)]
struct HUFF_SETUP {
    table: *const HUFF_ELEMENT,
    linbits: c_int,
    ncase: c_int,
}

const no_bits: c_int = 0;
const one_shot: c_int = 1;
const no_linbits: c_int = 2;
const have_linbits: c_int = 3;
const quad_a: c_int = 4;
const quad_b: c_int = 5;

const table_look: [HUFF_SETUP; 34] = [
    HUFF_SETUP { table: huff_table_0.as_ptr(), linbits: 0, ncase: no_bits },
    HUFF_SETUP { table: huff_table_1.as_ptr(), linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: huff_table_2.as_ptr(), linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: huff_table_3.as_ptr(), linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: huff_table_0.as_ptr(), linbits: 0, ncase: no_bits },
    HUFF_SETUP { table: huff_table_5.as_ptr(), linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: huff_table_6.as_ptr(), linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: huff_table_7.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_8.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_9.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_10.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_11.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_12.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_13.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_0.as_ptr(), linbits: 0, ncase: no_bits },
    HUFF_SETUP { table: huff_table_15.as_ptr(), linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 1, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 2, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 3, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 4, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 6, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 8, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 10, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_16.as_ptr(), linbits: 13, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 4, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 5, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 6, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 7, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 8, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 9, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 11, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_24.as_ptr(), linbits: 13, ncase: have_linbits },
    HUFF_SETUP { table: huff_table_0.as_ptr(), linbits: 0, ncase: quad_a },
    HUFF_SETUP { table: huff_table_0.as_ptr(), linbits: 0, ncase: quad_b },
];

/*========================================================*/
unsafe extern "C" {
    static mut bitdat: BITDAT;
}

/*----- get n bits  - checks for n+2 avail bits (linbits+sign) -----*/
unsafe fn bitget_lb(n: c_int) -> c_uint {
    let x: c_uint;

    unsafe {
        if bitdat.bits < (n + 2) {
            /* refill bit buf if necessary */
            while bitdat.bits <= 24 {
                bitdat.bitbuf = (bitdat.bitbuf << 8) | *bitdat.bs_ptr as c_uint;
                bitdat.bs_ptr = bitdat.bs_ptr.add(1);
                bitdat.bits += 8;
            }
        }
        bitdat.bits -= n;
        x = bitdat.bitbuf >> bitdat.bits;
        bitdat.bitbuf = bitdat.bitbuf.wrapping_sub(x << bitdat.bits);
    }
    x
}

/*------------- get n bits but DO NOT remove from bitstream --*/
unsafe fn bitget2(n: c_int) -> c_uint {
    let x: c_uint;

    unsafe {
        if bitdat.bits < (MAXBITS + 2) {
            /* refill bit buf if necessary */
            while bitdat.bits <= 24 {
                bitdat.bitbuf = (bitdat.bitbuf << 8) | *bitdat.bs_ptr as c_uint;
                bitdat.bs_ptr = bitdat.bs_ptr.add(1);
                bitdat.bits += 8;
            }
        }
        x = bitdat.bitbuf >> (bitdat.bits - n);
    }
    x
}

/*========================================================*/
/*========================================================*/
unsafe fn mac_bitget_check(n: c_int) {
    unsafe {
        if bitdat.bits < n {
            while bitdat.bits <= 24 {
                bitdat.bitbuf = (bitdat.bitbuf << 8) | *bitdat.bs_ptr as c_uint;
                bitdat.bs_ptr = bitdat.bs_ptr.add(1);
                bitdat.bits += 8;
            }
        }
    }
}

/*---------------------------------------------------------*/
unsafe fn mac_bitget2(n: c_int) -> c_int {
    unsafe { (bitdat.bitbuf >> (bitdat.bits - n)) as c_int }
}

/*---------------------------------------------------------*/
unsafe fn mac_bitget(n: c_int, code: *mut c_int) -> c_int {
    unsafe {
        bitdat.bits -= n;
        *code = (bitdat.bitbuf >> bitdat.bits) as c_int;
        bitdat.bitbuf = bitdat.bitbuf.wrapping_sub((*code as c_uint) << bitdat.bits);
        *code
    }
}

/*---------------------------------------------------------*/
unsafe fn mac_bitget_purge(n: c_int) {
    unsafe {
        bitdat.bits -= n;
        bitdat.bitbuf = bitdat
            .bitbuf
            .wrapping_sub((bitdat.bitbuf >> bitdat.bits) << bitdat.bits);
    }
}

/*---------------------------------------------------------*/
unsafe fn mac_bitget_1bit(code: *mut c_int) -> c_int {
    unsafe {
        bitdat.bits -= 1;
        *code = (bitdat.bitbuf >> bitdat.bits) as c_int;
        bitdat.bitbuf = bitdat.bitbuf.wrapping_sub((*code as c_uint) << bitdat.bits);
        *code
    }
}

/*========================================================*/
/*========================================================*/
pub unsafe extern "C" fn unpack_huff(xy: *mut [c_int; 2], mut n: c_int, ntable: c_int) {
    let mut i: c_int;
    let mut t: *const HUFF_ELEMENT;
    let t0: *const HUFF_ELEMENT;
    let linbits: c_int;
    let mut bits: c_int;
    let mut code: c_int = 0;
    let mut x: c_int;
    let mut y: c_int;

    if n <= 0 {
        return;
    }
    n = n >> 1; /* huff in pairs */
    /*-------------*/
    unsafe {
        t0 = table_look[ntable as usize].table;
        linbits = table_look[ntable as usize].linbits;
        match table_look[ntable as usize].ncase {
            /*------------------------------------------*/
            no_bits => {
                /*- table 0, no data, x=y=0--*/
                i = 0;
                while i < n {
                    (*xy.add(i as usize))[0] = 0;
                    (*xy.add(i as usize))[1] = 0;
                    i += 1;
                }
                return;
            }
            /*------------------------------------------*/
            one_shot => {
                /*- single lookup, no escapes -*/
                i = 0;
                while i < n {
                    mac_bitget_check(MAXBITS + 2);
                    bits = (*t0.add(0)).b.signbits as c_int;
                    code = mac_bitget2(bits);
                    mac_bitget_purge((*t0.add((1 + code) as usize)).b.purgebits as c_int);
                    x = (*t0.add((1 + code) as usize)).b.x as c_int;
                    y = (*t0.add((1 + code) as usize)).b.y as c_int;
                    if x != 0 {
                        if mac_bitget_1bit(&mut code) != 0 {
                            x = -x;
                        }
                    }
                    if y != 0 {
                        if mac_bitget_1bit(&mut code) != 0 {
                            y = -y;
                        }
                    }
                    (*xy.add(i as usize))[0] = x;
                    (*xy.add(i as usize))[1] = y;
                    if bitdat.bs_ptr > bitdat.bs_ptr_end {
                        break; /* bad data protect */
                    }
                    i += 1;
                }
                return;
            }
            /*------------------------------------------*/
            no_linbits => {
                i = 0;
                while i < n {
                    t = t0;
                    loop {
                        mac_bitget_check(MAXBITS + 2);
                        bits = (*t.add(0)).b.signbits as c_int;
                        code = mac_bitget2(bits);
                        if (*t.add((1 + code) as usize)).b.purgebits != 0 {
                            break;
                        }
                        t = t.add((*t.add((1 + code) as usize)).ptr as usize); /* ptr include 1+code */
                        mac_bitget_purge(bits);
                    }
                    mac_bitget_purge((*t.add((1 + code) as usize)).b.purgebits as c_int);
                    x = (*t.add((1 + code) as usize)).b.x as c_int;
                    y = (*t.add((1 + code) as usize)).b.y as c_int;
                    if x != 0 {
                        if mac_bitget_1bit(&mut code) != 0 {
                            x = -x;
                        }
                    }
                    if y != 0 {
                        if mac_bitget_1bit(&mut code) != 0 {
                            y = -y;
                        }
                    }
                    (*xy.add(i as usize))[0] = x;
                    (*xy.add(i as usize))[1] = y;
                    if bitdat.bs_ptr > bitdat.bs_ptr_end {
                        break; /* bad data protect */
                    }
                    i += 1;
                }
                return;
            }
            /*------------------------------------------*/
            have_linbits => {
                i = 0;
                while i < n {
                    t = t0;
                    loop {
                        bits = (*t.add(0)).b.signbits as c_int;
                        code = bitget2(bits) as c_int;
                        if (*t.add((1 + code) as usize)).b.purgebits != 0 {
                            break;
                        }
                        t = t.add((*t.add((1 + code) as usize)).ptr as usize); /* ptr includes 1+code */
                        mac_bitget_purge(bits);
                    }
                    mac_bitget_purge((*t.add((1 + code) as usize)).b.purgebits as c_int);
                    x = (*t.add((1 + code) as usize)).b.x as c_int;
                    y = (*t.add((1 + code) as usize)).b.y as c_int;
                    if x == 15 {
                        x += bitget_lb(linbits) as c_int;
                    }
                    if x != 0 {
                        if mac_bitget_1bit(&mut code) != 0 {
                            x = -x;
                        }
                    }
                    if y == 15 {
                        y += bitget_lb(linbits) as c_int;
                    }
                    if y != 0 {
                        if mac_bitget_1bit(&mut code) != 0 {
                            y = -y;
                        }
                    }
                    (*xy.add(i as usize))[0] = x;
                    (*xy.add(i as usize))[1] = y;
                    if bitdat.bs_ptr > bitdat.bs_ptr_end {
                        break; /* bad data protect */
                    }
                    i += 1;
                }
                return;
            }
            _ => {
                /*- table 0, no data, x=y=0--*/
                i = 0;
                while i < n {
                    (*xy.add(i as usize))[0] = 0;
                    (*xy.add(i as usize))[1] = 0;
                    i += 1;
                }
                return;
            }
        }
    }
    /*--- end switch ---*/
}

/*==========================================================*/
pub unsafe extern "C" fn unpack_huff_quad(
    vwxy: *mut [c_int; 4],
    mut n: c_int,
    mut nbits: c_int,
    ntable: c_int,
) -> c_int {
    let mut i: c_int;
    let mut code: c_int = 0;
    let mut x: c_int;
    let mut y: c_int;
    let mut v: c_int;
    let mut w: c_int;
    let mut tmp: c_int;
    let mut i_non_zero: c_int;
    let mut tmp_nz: c_int;

    tmp_nz = 15;
    i_non_zero = -1;

    n = n >> 2; /* huff in quads */

    if ntable != 0 {
        unsafe {
            i = 0;
            while i < n {
                if nbits < 4 {
                    break;
                }
                nbits -= 4;
                mac_bitget_check(8);
                tmp = mac_bitget(4, &mut code) ^ 15; /* one's complement of bitstream */
                if tmp != 0 {
                    i_non_zero = i;
                    tmp_nz = tmp;
                }
                v = (tmp >> 3) & 1;
                w = (tmp >> 2) & 1;
                x = (tmp >> 1) & 1;
                y = tmp & 1;
                if v != 0 {
                    if mac_bitget_1bit(&mut code) != 0 {
                        v = -v;
                    }
                    nbits -= 1;
                }
                if w != 0 {
                    if mac_bitget_1bit(&mut code) != 0 {
                        w = -w;
                    }
                    nbits -= 1;
                }
                if x != 0 {
                    if mac_bitget_1bit(&mut code) != 0 {
                        x = -x;
                    }
                    nbits -= 1;
                }
                if y != 0 {
                    if mac_bitget_1bit(&mut code) != 0 {
                        y = -y;
                    }
                    nbits -= 1;
                }
                (*vwxy.add(i as usize))[0] = v;
                (*vwxy.add(i as usize))[1] = w;
                (*vwxy.add(i as usize))[2] = x;
                (*vwxy.add(i as usize))[3] = y;
                if bitdat.bs_ptr > bitdat.bs_ptr_end {
                    break; /* bad data protect */
                }
                i += 1;
            }
            if nbits < 0 {
                i -= 1;
                (*vwxy.add(i as usize))[0] = 0;
                (*vwxy.add(i as usize))[1] = 0;
                (*vwxy.add(i as usize))[2] = 0;
                (*vwxy.add(i as usize))[3] = 0;
            }

            i_non_zero = (i_non_zero + 1) << 2;

            if (tmp_nz & 3) == 0 {
                i_non_zero -= 2;
            }

            return i_non_zero; /* return non-zero sample (to nearest pair) */
        }
    }

    /* case_quad_a: */
    unsafe {
        i = 0;
        while i < n {
            if nbits <= 0 {
                break;
            }
            mac_bitget_check(10);
            code = mac_bitget2(6);
            nbits -= quad_table_a[code as usize][0] as c_int;
            mac_bitget_purge(quad_table_a[code as usize][0] as c_int);
            tmp = quad_table_a[code as usize][1] as c_int;
            if tmp != 0 {
                i_non_zero = i;
                tmp_nz = tmp;
            }
            v = (tmp >> 3) & 1;
            w = (tmp >> 2) & 1;
            x = (tmp >> 1) & 1;
            y = tmp & 1;
            if v != 0 {
                if mac_bitget_1bit(&mut code) != 0 {
                    v = -v;
                }
                nbits -= 1;
            }
            if w != 0 {
                if mac_bitget_1bit(&mut code) != 0 {
                    w = -w;
                }
                nbits -= 1;
            }
            if x != 0 {
                if mac_bitget_1bit(&mut code) != 0 {
                    x = -x;
                }
                nbits -= 1;
            }
            if y != 0 {
                if mac_bitget_1bit(&mut code) != 0 {
                    y = -y;
                }
                nbits -= 1;
            }
            (*vwxy.add(i as usize))[0] = v;
            (*vwxy.add(i as usize))[1] = w;
            (*vwxy.add(i as usize))[2] = x;
            (*vwxy.add(i as usize))[3] = y;
            if bitdat.bs_ptr > bitdat.bs_ptr_end {
                break; /* bad data protect */
            }
            i += 1;
        }
        if i != 0 && nbits < 0 {
            i -= 1;
            (*vwxy.add(i as usize))[0] = 0;
            (*vwxy.add(i as usize))[1] = 0;
            (*vwxy.add(i as usize))[2] = 0;
            (*vwxy.add(i as usize))[3] = 0;
        }

        i_non_zero = (i_non_zero + 1) << 2;

        if (tmp_nz & 3) == 0 {
            i_non_zero -= 2;
        }

        return i_non_zero;
    }
}
/*-----------------------------------------------------*/
