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

use core::ffi::c_uchar;

// max bits required for any lookup - change if htable changes
// quad required 10 bit w/signs  must have (MAXBITS+2) >= 10
const MAXBITS: i32 = 9;

#[repr(C)]
pub struct HUFF_ELEMENT_b {
    pub signbits: c_uchar,
    pub x: c_uchar,
    pub y: c_uchar,
    pub purgebits: c_uchar, // 0 = esc
}

#[repr(C)]
pub union HUFF_ELEMENT {
    pub ptr: i32,
    pub b: HUFF_ELEMENT_b,
}

#[repr(C)]
pub struct BITDAT {
    pub bitbuf: u32,
    pub bits: i32,
    pub bs_ptr: *mut c_uchar,
    pub bs_ptr0: *mut c_uchar,
    pub bs_ptr_end: *mut c_uchar,
}

// Huffman lookup tables from htable.h
extern "C" {
    pub static huff_table_1: [HUFF_ELEMENT; 9];
    pub static huff_table_2: [HUFF_ELEMENT; 64];
    pub static huff_table_3: [HUFF_ELEMENT; 64];
    pub static huff_table_5: [HUFF_ELEMENT; 100];
    pub static huff_table_6: [HUFF_ELEMENT; 128];
    pub static huff_table_7: [HUFF_ELEMENT; 256];
    pub static huff_table_8: [HUFF_ELEMENT; 256];
    pub static huff_table_9: [HUFF_ELEMENT; 256];
    pub static huff_table_10: [HUFF_ELEMENT; 256];
    pub static huff_table_11: [HUFF_ELEMENT; 256];
    pub static huff_table_12: [HUFF_ELEMENT; 256];
    pub static huff_table_13: [HUFF_ELEMENT; 256];
    pub static huff_table_15: [HUFF_ELEMENT; 256];
    pub static huff_table_16: [HUFF_ELEMENT; 256];
    pub static huff_table_24: [HUFF_ELEMENT; 256];
}

static huff_table_0: [HUFF_ELEMENT; 4] = [
    HUFF_ELEMENT { ptr: 0 },
    HUFF_ELEMENT { ptr: 0 },
    HUFF_ELEMENT { ptr: 0 },
    HUFF_ELEMENT { ptr: 64 },
];

// 6 bit lookup (purgebits, value)
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
    linbits: i32,
    ncase: i32,
}

const no_bits: i32 = 0;
const one_shot: i32 = 1;
const no_linbits: i32 = 2;
const have_linbits: i32 = 3;
const quad_a: i32 = 4;
const quad_b: i32 = 5;

static table_look: [HUFF_SETUP; 33] = [
    HUFF_SETUP { table: &huff_table_0 as *const [HUFF_ELEMENT; 4] as *const HUFF_ELEMENT, linbits: 0, ncase: no_bits },
    HUFF_SETUP { table: &huff_table_1 as *const [HUFF_ELEMENT; 9] as *const HUFF_ELEMENT, linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: &huff_table_2 as *const [HUFF_ELEMENT; 64] as *const HUFF_ELEMENT, linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: &huff_table_3 as *const [HUFF_ELEMENT; 64] as *const HUFF_ELEMENT, linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: &huff_table_0 as *const [HUFF_ELEMENT; 4] as *const HUFF_ELEMENT, linbits: 0, ncase: no_bits },
    HUFF_SETUP { table: &huff_table_5 as *const [HUFF_ELEMENT; 100] as *const HUFF_ELEMENT, linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: &huff_table_6 as *const [HUFF_ELEMENT; 128] as *const HUFF_ELEMENT, linbits: 0, ncase: one_shot },
    HUFF_SETUP { table: &huff_table_7 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_8 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_9 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_10 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_11 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_12 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_13 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_0 as *const [HUFF_ELEMENT; 4] as *const HUFF_ELEMENT, linbits: 0, ncase: no_bits },
    HUFF_SETUP { table: &huff_table_15 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 0, ncase: no_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 1, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 2, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 3, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 4, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 6, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 8, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 10, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_16 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 13, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 4, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 5, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 6, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 7, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 8, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 9, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 11, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_24 as *const [HUFF_ELEMENT; 256] as *const HUFF_ELEMENT, linbits: 13, ncase: have_linbits },
    HUFF_SETUP { table: &huff_table_0 as *const [HUFF_ELEMENT; 4] as *const HUFF_ELEMENT, linbits: 0, ncase: quad_a },
    HUFF_SETUP { table: &huff_table_0 as *const [HUFF_ELEMENT; 4] as *const HUFF_ELEMENT, linbits: 0, ncase: quad_b },
];

extern "C" {
    pub static mut bitdat: BITDAT;
}

// ----- get n bits  - checks for n+2 avail bits (linbits+sign) -----
fn bitget_lb(bitdat: &mut BITDAT, n: i32) -> u32 {
    let mut x: u32;

    if bitdat.bits < (n + 2) {
        // refill bit buf if necessary
        while bitdat.bits <= 24 {
            unsafe {
                bitdat.bitbuf = (bitdat.bitbuf << 8) | (*bitdat.bs_ptr) as u32;
                bitdat.bs_ptr = bitdat.bs_ptr.offset(1);
            }
            bitdat.bits += 8;
        }
    }
    bitdat.bits -= n;
    x = bitdat.bitbuf >> (bitdat.bits as u32);
    bitdat.bitbuf -= x << (bitdat.bits as u32);
    x
}

// ------------- get n bits but DO NOT remove from bitstream --
fn bitget2(bitdat: &mut BITDAT, n: i32) -> u32 {
    let mut x: u32;

    if bitdat.bits < (MAXBITS + 2) {
        // refill bit buf if necessary
        while bitdat.bits <= 24 {
            unsafe {
                bitdat.bitbuf = (bitdat.bitbuf << 8) | (*bitdat.bs_ptr) as u32;
                bitdat.bs_ptr = bitdat.bs_ptr.offset(1);
            }
            bitdat.bits += 8;
        }
    }
    x = bitdat.bitbuf >> ((bitdat.bits - n) as u32);
    x
}

// Macro helpers
fn mac_bitget_check(bitdat: &mut BITDAT, n: i32) {
    if bitdat.bits < n {
        while bitdat.bits <= 24 {
            unsafe {
                bitdat.bitbuf = (bitdat.bitbuf << 8) | (*bitdat.bs_ptr) as u32;
                bitdat.bs_ptr = bitdat.bs_ptr.offset(1);
            }
            bitdat.bits += 8;
        }
    }
}

fn mac_bitget2(bitdat: &BITDAT, n: i32) -> u32 {
    bitdat.bitbuf >> ((bitdat.bits - n) as u32)
}

fn mac_bitget(bitdat: &mut BITDAT, n: i32) -> u32 {
    bitdat.bits -= n;
    let code = bitdat.bitbuf >> (bitdat.bits as u32);
    bitdat.bitbuf -= code << (bitdat.bits as u32);
    code
}

fn mac_bitget_purge(bitdat: &mut BITDAT, n: i32) {
    bitdat.bits -= n;
    bitdat.bitbuf -= (bitdat.bitbuf >> (bitdat.bits as u32)) << (bitdat.bits as u32);
}

fn mac_bitget_1bit(bitdat: &mut BITDAT) -> u32 {
    bitdat.bits -= 1;
    let code = bitdat.bitbuf >> (bitdat.bits as u32);
    bitdat.bitbuf -= code << (bitdat.bits as u32);
    code
}

pub fn unpack_huff(xy: &mut [[i32; 2]], n: i32, ntable: i32) {
    let mut i: i32;
    let mut t: *const HUFF_ELEMENT;
    let t0: *const HUFF_ELEMENT;
    let linbits: i32;
    let mut bits: i32;
    let mut code: u32;
    let mut x: i32;
    let mut y: i32;

    if n <= 0 {
        return;
    }

    unsafe {
        let mut n_local = n >> 1; // huff in pairs

        t0 = table_look[ntable as usize].table;
        linbits = table_look[ntable as usize].linbits;
        match table_look[ntable as usize].ncase {
            no_bits => {
                // table 0, no data, x=y=0
                for i in 0..n_local {
                    xy[i as usize][0] = 0;
                    xy[i as usize][1] = 0;
                }
                return;
            }
            one_shot => {
                // single lookup, no escapes
                for i in 0..n_local {
                    mac_bitget_check(&mut bitdat, MAXBITS + 2);
                    bits = (*t0).b.signbits as i32;
                    code = mac_bitget2(&bitdat, bits);
                    let code_offset = 1isize + code as isize;
                    mac_bitget_purge(&mut bitdat, (*t0.offset(code_offset)).b.purgebits as i32);
                    x = (*t0.offset(code_offset)).b.x as i32;
                    y = (*t0.offset(code_offset)).b.y as i32;
                    if x != 0 {
                        if mac_bitget_1bit(&mut bitdat) != 0 {
                            x = -x;
                        }
                    }
                    if y != 0 {
                        if mac_bitget_1bit(&mut bitdat) != 0 {
                            y = -y;
                        }
                    }
                    xy[i as usize][0] = x;
                    xy[i as usize][1] = y;
                    if bitdat.bs_ptr > bitdat.bs_ptr_end {
                        break; // bad data protect
                    }
                }
                return;
            }
            no_linbits => {
                for i in 0..n_local {
                    t = t0;
                    loop {
                        mac_bitget_check(&mut bitdat, MAXBITS + 2);
                        bits = (*t).b.signbits as i32;
                        code = mac_bitget2(&bitdat, bits);
                        let code_offset = 1isize + code as isize;
                        if (*t.offset(code_offset)).b.purgebits != 0 {
                            break;
                        }
                        t = t.offset((*t).ptr as isize);
                        mac_bitget_purge(&mut bitdat, bits);
                    }
                    let code_offset = 1isize + code as isize;
                    mac_bitget_purge(&mut bitdat, (*t.offset(code_offset)).b.purgebits as i32);
                    x = (*t.offset(code_offset)).b.x as i32;
                    y = (*t.offset(code_offset)).b.y as i32;
                    if x != 0 {
                        if mac_bitget_1bit(&mut bitdat) != 0 {
                            x = -x;
                        }
                    }
                    if y != 0 {
                        if mac_bitget_1bit(&mut bitdat) != 0 {
                            y = -y;
                        }
                    }
                    xy[i as usize][0] = x;
                    xy[i as usize][1] = y;
                    if bitdat.bs_ptr > bitdat.bs_ptr_end {
                        break; // bad data protect
                    }
                }
                return;
            }
            have_linbits => {
                for i in 0..n_local {
                    t = t0;
                    loop {
                        bits = (*t).b.signbits as i32;
                        code = bitget2(&bitdat, bits);
                        let code_offset = 1isize + code as isize;
                        if (*t.offset(code_offset)).b.purgebits != 0 {
                            break;
                        }
                        t = t.offset((*t).ptr as isize);
                        mac_bitget_purge(&mut bitdat, bits);
                    }
                    let code_offset = 1isize + code as isize;
                    mac_bitget_purge(&mut bitdat, (*t.offset(code_offset)).b.purgebits as i32);
                    x = (*t.offset(code_offset)).b.x as i32;
                    y = (*t.offset(code_offset)).b.y as i32;
                    if x == 15 {
                        x += bitget_lb(&mut bitdat, linbits) as i32;
                    }
                    if x != 0 {
                        if mac_bitget_1bit(&mut bitdat) != 0 {
                            x = -x;
                        }
                    }
                    if y == 15 {
                        y += bitget_lb(&mut bitdat, linbits) as i32;
                    }
                    if y != 0 {
                        if mac_bitget_1bit(&mut bitdat) != 0 {
                            y = -y;
                        }
                    }
                    xy[i as usize][0] = x;
                    xy[i as usize][1] = y;
                    if bitdat.bs_ptr > bitdat.bs_ptr_end {
                        break; // bad data protect
                    }
                }
                return;
            }
            _ => {}
        }
    }
}

pub fn unpack_huff_quad(
    vwxy: &mut [[i32; 4]],
    n: i32,
    nbits: i32,
    ntable: i32,
) -> i32 {
    let mut i: i32;
    let mut code: u32;
    let mut x: i32;
    let mut y: i32;
    let mut v: i32;
    let mut w: i32;
    let mut tmp: i32;
    let mut i_non_zero: i32;
    let mut tmp_nz: i32;

    tmp_nz = 15;
    i_non_zero = -1;

    let mut n_local = n >> 2; // huff in quads
    let mut nbits_local = nbits;

    if ntable == 0 {
        // case_quad_a
        i = 0;
        unsafe {
            while i < n_local {
                if nbits_local <= 0 {
                    break;
                }
                mac_bitget_check(&mut bitdat, 10);
                code = mac_bitget2(&bitdat, 6);
                nbits_local -= quad_table_a[code as usize][0] as i32;
                mac_bitget_purge(&mut bitdat, quad_table_a[code as usize][0] as i32);
                tmp = quad_table_a[code as usize][1] as i32;
                if tmp != 0 {
                    i_non_zero = i;
                    tmp_nz = tmp;
                }
                v = (tmp >> 3) & 1;
                w = (tmp >> 2) & 1;
                x = (tmp >> 1) & 1;
                y = tmp & 1;
                if v != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        v = -v;
                    }
                    nbits_local -= 1;
                }
                if w != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        w = -w;
                    }
                    nbits_local -= 1;
                }
                if x != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        x = -x;
                    }
                    nbits_local -= 1;
                }
                if y != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        y = -y;
                    }
                    nbits_local -= 1;
                }
                vwxy[i as usize][0] = v;
                vwxy[i as usize][1] = w;
                vwxy[i as usize][2] = x;
                vwxy[i as usize][3] = y;
                if bitdat.bs_ptr > bitdat.bs_ptr_end {
                    break; // bad data protect
                }
                i += 1;
            }
        }
        if i != 0 && nbits_local < 0 {
            i -= 1;
            vwxy[i as usize][0] = 0;
            vwxy[i as usize][1] = 0;
            vwxy[i as usize][2] = 0;
            vwxy[i as usize][3] = 0;
        }

        i_non_zero = (i_non_zero + 1) << 2;

        if (tmp_nz & 3) == 0 {
            i_non_zero -= 2;
        }

        return i_non_zero;
    } else {
        // case_quad_b
        i = 0;
        unsafe {
            while i < n_local {
                if nbits_local < 4 {
                    break;
                }
                nbits_local -= 4;
                mac_bitget_check(&mut bitdat, 8);
                tmp = (mac_bitget(&mut bitdat, 4) ^ 15) as i32; // one's complement of bitstream
                if tmp != 0 {
                    i_non_zero = i;
                    tmp_nz = tmp;
                }
                v = (tmp >> 3) & 1;
                w = (tmp >> 2) & 1;
                x = (tmp >> 1) & 1;
                y = tmp & 1;
                if v != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        v = -v;
                    }
                    nbits_local -= 1;
                }
                if w != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        w = -w;
                    }
                    nbits_local -= 1;
                }
                if x != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        x = -x;
                    }
                    nbits_local -= 1;
                }
                if y != 0 {
                    if mac_bitget_1bit(&mut bitdat) != 0 {
                        y = -y;
                    }
                    nbits_local -= 1;
                }
                vwxy[i as usize][0] = v;
                vwxy[i as usize][1] = w;
                vwxy[i as usize][2] = x;
                vwxy[i as usize][3] = y;
                if bitdat.bs_ptr > bitdat.bs_ptr_end {
                    break; // bad data protect
                }
                i += 1;
            }
        }
        if nbits_local < 0 {
            i -= 1;
            vwxy[i as usize][0] = 0;
            vwxy[i as usize][1] = 0;
            vwxy[i as usize][2] = 0;
            vwxy[i as usize][3] = 0;
        }

        i_non_zero = (i_non_zero + 1) << 2;

        if (tmp_nz & 3) == 0 {
            i_non_zero -= 2;
        }

        return i_non_zero; // return non-zero sample (to nearest pair)
    }
}
