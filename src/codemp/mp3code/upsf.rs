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

    $Id: upsf.c,v 1.3 1999/10/19 07:13:09 elrod Exp $
____________________________________________________________________________*/

/****  upsf.c  ***************************************************

Layer III
    unpack scale factors



******************************************************************/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_uint};

use super::l3_h::{GR, IS_SF_INFO, SCALEFACT};

//extern int iframe;

unsafe extern "C" {
    pub fn bitget(n: c_int) -> c_uint;
}

/*------------------------------------------------------------*/
static slen_table: [[c_int; 2]; 16] = [
    [0, 0],
    [0, 1],
    [0, 2],
    [0, 3],
    [3, 0],
    [1, 1],
    [1, 2],
    [1, 3],
    [2, 1],
    [2, 2],
    [2, 3],
    [3, 1],
    [3, 2],
    [3, 3],
    [4, 2],
    [4, 3],
];

/* nr_table[size+3*is_right][block type 0,1,3  2, 2+mixed][4]  */
/* for bt=2 nr is count for group of 3 */
static nr_table: [[[c_int; 4]; 3]; 6] = [
    [[6, 5, 5, 5], [3, 3, 3, 3], [6, 3, 3, 3]],
    [[6, 5, 7, 3], [3, 3, 4, 2], [6, 3, 4, 2]],
    [[11, 10, 0, 0], [6, 6, 0, 0], [6, 3, 6, 0]], /* adjusted *//* 15, 18, 0, 0,   */
    /*-intensity stereo right chan--*/
    [[7, 7, 7, 0], [4, 4, 4, 0], [6, 5, 4, 0]],
    [[6, 6, 6, 3], [4, 3, 3, 2], [6, 4, 3, 2]],
    [[8, 8, 5, 0], [5, 4, 3, 0], [6, 6, 3, 0]],
];

/*=============================================================*/
pub unsafe extern "C" fn unpack_sf_sub_MPEG1(
    sf: *mut SCALEFACT,
    grdat: *mut GR,
    scfsi: c_int, /* bit flag */
    gr: c_int,
) {
    let mut sfb: c_int;
    let slen0: c_int;
    let slen1: c_int;
    let block_type: c_int;
    let mixed_block_flag: c_int;
    let scalefac_compress: c_int;

    block_type = (*grdat).block_type;
    mixed_block_flag = (*grdat).mixed_block_flag;
    scalefac_compress = (*grdat).scalefac_compress;

    slen0 = slen_table[scalefac_compress as usize][0];
    slen1 = slen_table[scalefac_compress as usize][1];

    if block_type == 2 {
        if mixed_block_flag != 0 {
            /* mixed */
            sfb = 0;
            while sfb < 8 {
                (*sf).l[sfb as usize] = bitget(slen0) as c_int;
                sfb += 1;
            }
            sfb = 3;
            while sfb < 6 {
                (*sf).s[0][sfb as usize] = bitget(slen0) as c_int;
                (*sf).s[1][sfb as usize] = bitget(slen0) as c_int;
                (*sf).s[2][sfb as usize] = bitget(slen0) as c_int;
                sfb += 1;
            }
            sfb = 6;
            while sfb < 12 {
                (*sf).s[0][sfb as usize] = bitget(slen1) as c_int;
                (*sf).s[1][sfb as usize] = bitget(slen1) as c_int;
                (*sf).s[2][sfb as usize] = bitget(slen1) as c_int;
                sfb += 1;
            }
            return;
        }
        sfb = 0;
        while sfb < 6 {
            (*sf).s[0][sfb as usize] = bitget(slen0) as c_int;
            (*sf).s[1][sfb as usize] = bitget(slen0) as c_int;
            (*sf).s[2][sfb as usize] = bitget(slen0) as c_int;
            sfb += 1;
        }
        while sfb < 12 {
            (*sf).s[0][sfb as usize] = bitget(slen1) as c_int;
            (*sf).s[1][sfb as usize] = bitget(slen1) as c_int;
            (*sf).s[2][sfb as usize] = bitget(slen1) as c_int;
            sfb += 1;
        }
        return;
    }

    /* long blocks types 0 1 3, first granule */
    if gr == 0 {
        sfb = 0;
        while sfb < 11 {
            (*sf).l[sfb as usize] = bitget(slen0) as c_int;
            sfb += 1;
        }
        while sfb < 21 {
            (*sf).l[sfb as usize] = bitget(slen1) as c_int;
            sfb += 1;
        }
        return;
    }

    /* long blocks 0, 1, 3, second granule */
    sfb = 0;
    if (scfsi & 8) != 0 {
        while sfb < 6 {
            (*sf).l[sfb as usize] = (*sf.offset(-2)).l[sfb as usize];
            sfb += 1;
        }
    } else {
        while sfb < 6 {
            (*sf).l[sfb as usize] = bitget(slen0) as c_int;
            sfb += 1;
        }
    }
    if (scfsi & 4) != 0 {
        while sfb < 11 {
            (*sf).l[sfb as usize] = (*sf.offset(-2)).l[sfb as usize];
            sfb += 1;
        }
    } else {
        while sfb < 11 {
            (*sf).l[sfb as usize] = bitget(slen0) as c_int;
            sfb += 1;
        }
    }
    if (scfsi & 2) != 0 {
        while sfb < 16 {
            (*sf).l[sfb as usize] = (*sf.offset(-2)).l[sfb as usize];
            sfb += 1;
        }
    } else {
        while sfb < 16 {
            (*sf).l[sfb as usize] = bitget(slen1) as c_int;
            sfb += 1;
        }
    }
    if (scfsi & 1) != 0 {
        while sfb < 21 {
            (*sf).l[sfb as usize] = (*sf.offset(-2)).l[sfb as usize];
            sfb += 1;
        }
    } else {
        while sfb < 21 {
            (*sf).l[sfb as usize] = bitget(slen1) as c_int;
            sfb += 1;
        }
    }

    return;
}

/*=============================================================*/
pub unsafe extern "C" fn unpack_sf_sub_MPEG2(
    sf: *mut SCALEFACT,
    grdat: *mut GR,
    is_and_ch: c_int,
    sf_info: *mut IS_SF_INFO,
) {
    let mut sfb: c_int;
    let mut slen1: c_int;
    let mut slen2: c_int;
    let mut slen3: c_int;
    let mut slen4: c_int;
    let nr1: c_int;
    let nr2: c_int;
    let nr3: c_int;
    let nr4: c_int;
    let mut i: c_int;
    let k: c_int;
    let mut preflag: c_int;
    let mut intensity_scale: c_int;
    let block_type: c_int;
    let mixed_block_flag: c_int;
    let mut scalefac_compress: c_int;

    block_type = (*grdat).block_type;
    mixed_block_flag = (*grdat).mixed_block_flag;
    scalefac_compress = (*grdat).scalefac_compress;

    preflag = 0;
    intensity_scale = 0; /* to avoid compiler warning */
    if is_and_ch == 0 {
        if scalefac_compress < 400 {
            slen2 = scalefac_compress >> 4;
            slen1 = slen2 / 5;
            slen2 = slen2 % 5;
            slen4 = scalefac_compress & 15;
            slen3 = slen4 >> 2;
            slen4 = slen4 & 3;
            k = 0;
        } else if scalefac_compress < 500 {
            scalefac_compress -= 400;
            slen2 = scalefac_compress >> 2;
            slen1 = slen2 / 5;
            slen2 = slen2 % 5;
            slen3 = scalefac_compress & 3;
            slen4 = 0;
            k = 1;
        } else {
            scalefac_compress -= 500;
            slen1 = scalefac_compress / 3;
            slen2 = scalefac_compress % 3;
            slen4 = 0;
            slen3 = slen4;
            if mixed_block_flag != 0 {
                slen3 = slen2; /* adjust for long/short mix logic */
                slen2 = slen1;
            }
            preflag = 1;
            k = 2;
        }
    } else {
        /* intensity stereo ch = 1 (right) */
        intensity_scale = scalefac_compress & 1;
        scalefac_compress >>= 1;
        if scalefac_compress < 180 {
            slen1 = scalefac_compress / 36;
            slen2 = scalefac_compress % 36;
            slen3 = slen2 % 6;
            slen2 = slen2 / 6;
            slen4 = 0;
            k = 3 + 0;
        } else if scalefac_compress < 244 {
            scalefac_compress -= 180;
            slen3 = scalefac_compress & 3;
            scalefac_compress >>= 2;
            slen2 = scalefac_compress & 3;
            slen1 = scalefac_compress >> 2;
            slen4 = 0;
            k = 3 + 1;
        } else {
            scalefac_compress -= 244;
            slen1 = scalefac_compress / 3;
            slen2 = scalefac_compress % 3;
            slen4 = 0;
            slen3 = slen4;
            k = 3 + 2;
        }
    }

    i = 0;
    if block_type == 2 {
        i = (mixed_block_flag & 1) + 1;
    }
    nr1 = nr_table[k as usize][i as usize][0];
    nr2 = nr_table[k as usize][i as usize][1];
    nr3 = nr_table[k as usize][i as usize][2];
    nr4 = nr_table[k as usize][i as usize][3];

    /* return is scale factor info (for right chan is mode) */
    if is_and_ch != 0 {
        (*sf_info).nr[0] = nr1;
        (*sf_info).nr[1] = nr2;
        (*sf_info).nr[2] = nr3;
        (*sf_info).slen[0] = slen1;
        (*sf_info).slen[1] = slen2;
        (*sf_info).slen[2] = slen3;
        (*sf_info).intensity_scale = intensity_scale;
    }
    (*grdat).preflag = preflag; /* return preflag */

    /*--------------------------------------*/
    if block_type == 2 {
        if mixed_block_flag != 0 {
            /* mixed */
            if slen1 != 0 {
                /* long block portion */
                sfb = 0;
                while sfb < 6 {
                    (*sf).l[sfb as usize] = bitget(slen1) as c_int;
                    sfb += 1;
                }
            } else {
                sfb = 0;
                while sfb < 6 {
                    (*sf).l[sfb as usize] = 0;
                    sfb += 1;
                }
            }
            sfb = 3; /* start sfb for short */
        } else {
            /* all short, initial short blocks */
            sfb = 0;
            if slen1 != 0 {
                i = 0;
                while i < nr1 {
                    (*sf).s[0][sfb as usize] = bitget(slen1) as c_int;
                    (*sf).s[1][sfb as usize] = bitget(slen1) as c_int;
                    (*sf).s[2][sfb as usize] = bitget(slen1) as c_int;
                    i += 1;
                    sfb += 1;
                }
            } else {
                i = 0;
                while i < nr1 {
                    (*sf).s[0][sfb as usize] = 0;
                    (*sf).s[1][sfb as usize] = 0;
                    (*sf).s[2][sfb as usize] = 0;
                    i += 1;
                    sfb += 1;
                }
            }
        }
        /* remaining short blocks */
        if slen2 != 0 {
            i = 0;
            while i < nr2 {
                (*sf).s[0][sfb as usize] = bitget(slen2) as c_int;
                (*sf).s[1][sfb as usize] = bitget(slen2) as c_int;
                (*sf).s[2][sfb as usize] = bitget(slen2) as c_int;
                i += 1;
                sfb += 1;
            }
        } else {
            i = 0;
            while i < nr2 {
                (*sf).s[0][sfb as usize] = 0;
                (*sf).s[1][sfb as usize] = 0;
                (*sf).s[2][sfb as usize] = 0;
                i += 1;
                sfb += 1;
            }
        }
        if slen3 != 0 {
            i = 0;
            while i < nr3 {
                (*sf).s[0][sfb as usize] = bitget(slen3) as c_int;
                (*sf).s[1][sfb as usize] = bitget(slen3) as c_int;
                (*sf).s[2][sfb as usize] = bitget(slen3) as c_int;
                i += 1;
                sfb += 1;
            }
        } else {
            i = 0;
            while i < nr3 {
                (*sf).s[0][sfb as usize] = 0;
                (*sf).s[1][sfb as usize] = 0;
                (*sf).s[2][sfb as usize] = 0;
                i += 1;
                sfb += 1;
            }
        }
        if slen4 != 0 {
            i = 0;
            while i < nr4 {
                (*sf).s[0][sfb as usize] = bitget(slen4) as c_int;
                (*sf).s[1][sfb as usize] = bitget(slen4) as c_int;
                (*sf).s[2][sfb as usize] = bitget(slen4) as c_int;
                i += 1;
                sfb += 1;
            }
        } else {
            i = 0;
            while i < nr4 {
                (*sf).s[0][sfb as usize] = 0;
                (*sf).s[1][sfb as usize] = 0;
                (*sf).s[2][sfb as usize] = 0;
                i += 1;
                sfb += 1;
            }
        }
        return;
    }

    /* long blocks types 0 1 3 */
    sfb = 0;
    if slen1 != 0 {
        i = 0;
        while i < nr1 {
            (*sf).l[sfb as usize] = bitget(slen1) as c_int;
            i += 1;
            sfb += 1;
        }
    } else {
        i = 0;
        while i < nr1 {
            (*sf).l[sfb as usize] = 0;
            i += 1;
            sfb += 1;
        }
    }

    if slen2 != 0 {
        i = 0;
        while i < nr2 {
            (*sf).l[sfb as usize] = bitget(slen2) as c_int;
            i += 1;
            sfb += 1;
        }
    } else {
        i = 0;
        while i < nr2 {
            (*sf).l[sfb as usize] = 0;
            i += 1;
            sfb += 1;
        }
    }

    if slen3 != 0 {
        i = 0;
        while i < nr3 {
            (*sf).l[sfb as usize] = bitget(slen3) as c_int;
            i += 1;
            sfb += 1;
        }
    } else {
        i = 0;
        while i < nr3 {
            (*sf).l[sfb as usize] = 0;
            i += 1;
            sfb += 1;
        }
    }

    if slen4 != 0 {
        i = 0;
        while i < nr4 {
            (*sf).l[sfb as usize] = bitget(slen4) as c_int;
            i += 1;
            sfb += 1;
        }
    } else {
        i = 0;
        while i < nr4 {
            (*sf).l[sfb as usize] = 0;
            i += 1;
            sfb += 1;
        }
    }
}
/*-------------------------------------------------*/
