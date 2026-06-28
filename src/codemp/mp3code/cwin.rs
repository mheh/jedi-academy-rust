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

    $Id: cwin.c,v 1.7 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  cwin.c  ***************************************************

include to cwinm.c

MPEG audio decoder, float window routines
portable C

******************************************************************/

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_long, c_short, c_uchar};

/*-------------------------------------------------------------------------*/
static wincoef: [f32; 256] = [0.0; 256];

/*-------------------------------------------------------------------------*/
pub unsafe fn window(vbuf: *mut f32, vb_ptr: c_int, mut pcm: *mut c_short) {
    let mut i: c_int;
    let mut j: c_int;
    let mut si: c_int;
    let mut bx: c_int;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: c_long;

    si = vb_ptr + 16;
    bx = (si + 32) & 511;
    coef = wincoef.as_ptr();

    /*-- first 16 --*/
    i = 0;
    while i < 16 {
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = (si + 64) & 511;
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        si += 1;
        bx -= 1;
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(1);
        i += 1;
    }
    /*--  special case --*/
    sum = 0.0f32;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = (bx + 64) & 511;
        j += 1;
    }
    tmp = sum as c_long;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as c_short;
    pcm = pcm.add(1);
    /*-- last 15 --*/
    coef = wincoef.as_ptr().add(255); /* back pass through coefs */
    i = 0;
    while i < 15 {
        si -= 1;
        bx += 1;
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = (si + 64) & 511;
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(1);
        i += 1;
    }
}

/*------------------------------------------------------------*/
pub unsafe fn window_dual(vbuf: *mut f32, vb_ptr: c_int, mut pcm: *mut c_short) {
    let mut i: c_int;
    let mut j: c_int; /* dual window interleaves output */
    let mut si: c_int;
    let mut bx: c_int;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: c_long;

    si = vb_ptr + 16;
    bx = (si + 32) & 511;
    coef = wincoef.as_ptr();

    /*-- first 16 --*/
    i = 0;
    while i < 16 {
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = (si + 64) & 511;
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        si += 1;
        bx -= 1;
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(2);
        i += 1;
    }
    /*--  special case --*/
    sum = 0.0f32;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = (bx + 64) & 511;
        j += 1;
    }
    tmp = sum as c_long;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as c_short;
    pcm = pcm.add(2);
    /*-- last 15 --*/
    coef = wincoef.as_ptr().add(255); /* back pass through coefs */
    i = 0;
    while i < 15 {
        si -= 1;
        bx += 1;
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = (si + 64) & 511;
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(2);
        i += 1;
    }
}

/*------------------------------------------------------------*/
/*------------------- 16 pt window ------------------------------*/
pub unsafe fn window16(vbuf: *mut f32, vb_ptr: c_int, mut pcm: *mut c_short) {
    let mut i: c_int;
    let mut j: c_int;
    let mut si: c_uchar;
    let mut bx: c_uchar;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: c_long;

    si = (vb_ptr + 8) as c_uchar;
    bx = si.wrapping_add(16);
    coef = wincoef.as_ptr();

    /*-- first 8 --*/
    i = 0;
    while i < 8 {
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = si.wrapping_add(32);
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = bx.wrapping_add(32);
            j += 1;
        }
        si = si.wrapping_add(1);
        bx = bx.wrapping_sub(1);
        coef = coef.add(16);
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(1);
        i += 1;
    }
    /*--  special case --*/
    sum = 0.0f32;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = bx.wrapping_add(32);
        j += 1;
    }
    tmp = sum as c_long;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as c_short;
    pcm = pcm.add(1);
    /*-- last 7 --*/
    coef = wincoef.as_ptr().add(255); /* back pass through coefs */
    i = 0;
    while i < 7 {
        coef = coef.sub(16);
        si = si.wrapping_sub(1);
        bx = bx.wrapping_add(1);
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = si.wrapping_add(32);
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = bx.wrapping_add(32);
            j += 1;
        }
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(1);
        i += 1;
    }
}

/*--------------- 16 pt dual window (interleaved output) -----------------*/
pub unsafe fn window16_dual(vbuf: *mut f32, vb_ptr: c_int, mut pcm: *mut c_short) {
    let mut i: c_int;
    let mut j: c_int;
    let mut si: c_uchar;
    let mut bx: c_uchar;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: c_long;

    si = (vb_ptr + 8) as c_uchar;
    bx = si.wrapping_add(16);
    coef = wincoef.as_ptr();

    /*-- first 8 --*/
    i = 0;
    while i < 8 {
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = si.wrapping_add(32);
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = bx.wrapping_add(32);
            j += 1;
        }
        si = si.wrapping_add(1);
        bx = bx.wrapping_sub(1);
        coef = coef.add(16);
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(2);
        i += 1;
    }
    /*--  special case --*/
    sum = 0.0f32;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = bx.wrapping_add(32);
        j += 1;
    }
    tmp = sum as c_long;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as c_short;
    pcm = pcm.add(2);
    /*-- last 7 --*/
    coef = wincoef.as_ptr().add(255); /* back pass through coefs */
    i = 0;
    while i < 7 {
        coef = coef.sub(16);
        si = si.wrapping_sub(1);
        bx = bx.wrapping_add(1);
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = si.wrapping_add(32);
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = bx.wrapping_add(32);
            j += 1;
        }
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(2);
        i += 1;
    }
}

/*------------------- 8 pt window ------------------------------*/
pub unsafe fn window8(vbuf: *mut f32, vb_ptr: c_int, mut pcm: *mut c_short) {
    let mut i: c_int;
    let mut j: c_int;
    let mut si: c_int;
    let mut bx: c_int;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: c_long;

    si = vb_ptr + 4;
    bx = (si + 8) & 127;
    coef = wincoef.as_ptr();

    /*-- first 4 --*/
    i = 0;
    while i < 4 {
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = (si + 16) & 127;
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        si += 1;
        bx -= 1;
        coef = coef.add(48);
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(1);
        i += 1;
    }
    /*--  special case --*/
    sum = 0.0f32;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = (bx + 16) & 127;
        j += 1;
    }
    tmp = sum as c_long;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as c_short;
    pcm = pcm.add(1);
    /*-- last 3 --*/
    coef = wincoef.as_ptr().add(255); /* back pass through coefs */
    i = 0;
    while i < 3 {
        coef = coef.sub(48);
        si -= 1;
        bx += 1;
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = (si + 16) & 127;
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(1);
        i += 1;
    }
}

/*--------------- 8 pt dual window (interleaved output) -----------------*/
pub unsafe fn window8_dual(vbuf: *mut f32, vb_ptr: c_int, mut pcm: *mut c_short) {
    let mut i: c_int;
    let mut j: c_int;
    let mut si: c_int;
    let mut bx: c_int;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: c_long;

    si = vb_ptr + 4;
    bx = (si + 8) & 127;
    coef = wincoef.as_ptr();

    /*-- first 4 --*/
    i = 0;
    while i < 4 {
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = (si + 16) & 127;
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        si += 1;
        bx -= 1;
        coef = coef.add(48);
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(2);
        i += 1;
    }
    /*--  special case --*/
    sum = 0.0f32;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = (bx + 16) & 127;
        j += 1;
    }
    tmp = sum as c_long;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as c_short;
    pcm = pcm.add(2);
    /*-- last 3 --*/
    coef = wincoef.as_ptr().add(255); /* back pass through coefs */
    i = 0;
    while i < 3 {
        coef = coef.sub(48);
        si -= 1;
        bx += 1;
        sum = 0.0f32;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = (si + 16) & 127;
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        tmp = sum as c_long;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as c_short;
        pcm = pcm.add(2);
        i += 1;
    }
}

/*------------------------------------------------------------*/
