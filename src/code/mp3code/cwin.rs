//! FreeAmp - The Free MP3 Player
//!
//! MP3 Decoder originally Copyright (C) 1995-1997 Xing Technology
//! Corp.  http://www.xingtech.com
//!
//! Portions Copyright (C) 1998-1999 EMusic.com
//!
//! This program is free software; you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation; either version 2 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program; if not, write to the Free Software
//! Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
//!
//! $Id: cwin.c,v 1.7 1999/10/19 07:13:08 elrod Exp $
//!
//! cwin.c
//!
//! include to cwinm.c
//!
//! MPEG audio decoder, float window routines
//! portable C

extern "C" {
    static wincoef: *const f32;
}

//  -----------------------------------------------------------------------
pub unsafe fn window(vbuf: *const f32, mut vb_ptr: i32, mut pcm: *mut i16) {
    let mut i: i32;
    let mut j: i32;
    let mut si: i32;
    let mut bx: i32;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: i64;

    si = vb_ptr + 16;
    bx = (si + 32) & 511;
    coef = wincoef;

    //-- first 16 --
    i = 0;
    while i < 16 {
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 511));
            coef = coef.add(1);
            si = (si + 64) & 511;
            sum -= (*coef) * (*vbuf.add((bx as usize) & 511));
            coef = coef.add(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        si += 1;
        bx -= 1;
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(1);
        i += 1;
    }
    //--  special case --
    sum = 0.0;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add((bx as usize) & 511));
        coef = coef.add(1);
        bx = (bx + 64) & 511;
        j += 1;
    }
    tmp = sum as i64;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as i16;
    pcm = pcm.add(1);
    //-- last 15 --
    coef = wincoef.add(255); /* back pass through coefs */
    i = 0;
    while i < 15 {
        si -= 1;
        bx += 1;
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 511));
            coef = coef.sub(1);
            si = (si + 64) & 511;
            sum += (*coef) * (*vbuf.add((bx as usize) & 511));
            coef = coef.sub(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(1);
        i += 1;
    }
}

//  ---------------------------------------------------------------
pub unsafe fn window_dual(vbuf: *const f32, mut vb_ptr: i32, mut pcm: *mut i16) {
    let mut i: i32;
    let mut j: i32; /* dual window interleaves output */
    let mut si: i32;
    let mut bx: i32;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: i64;

    si = vb_ptr + 16;
    bx = (si + 32) & 511;
    coef = wincoef;

    //-- first 16 --
    i = 0;
    while i < 16 {
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 511));
            coef = coef.add(1);
            si = (si + 64) & 511;
            sum -= (*coef) * (*vbuf.add((bx as usize) & 511));
            coef = coef.add(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        si += 1;
        bx -= 1;
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(2);
        i += 1;
    }
    //--  special case --
    sum = 0.0;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add((bx as usize) & 511));
        coef = coef.add(1);
        bx = (bx + 64) & 511;
        j += 1;
    }
    tmp = sum as i64;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as i16;
    pcm = pcm.add(2);
    //-- last 15 --
    coef = wincoef.add(255); /* back pass through coefs */
    i = 0;
    while i < 15 {
        si -= 1;
        bx += 1;
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 511));
            coef = coef.sub(1);
            si = (si + 64) & 511;
            sum += (*coef) * (*vbuf.add((bx as usize) & 511));
            coef = coef.sub(1);
            bx = (bx + 64) & 511;
            j += 1;
        }
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(2);
        i += 1;
    }
}
//  ---------------------------------------------------------------
//  ------------------- 16 pt window --------------------------------
pub unsafe fn window16(vbuf: *const f32, mut vb_ptr: i32, mut pcm: *mut i16) {
    let mut i: i32;
    let mut j: i32;
    let mut si: u8;
    let mut bx: u8;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: i64;

    si = (vb_ptr + 8) as u8;
    bx = (si as i32 + 16) as u8;
    coef = wincoef;

    //-- first 8 --
    i = 0;
    while i < 8 {
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = ((si as i32 + 32) as u8);
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = ((bx as i32 + 32) as u8);
            j += 1;
        }
        si = ((si as i32 + 1) as u8);
        bx = ((bx as i32 - 1) as u8);
        coef = coef.add(16);
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(1);
        i += 1;
    }
    //--  special case --
    sum = 0.0;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = ((bx as i32 + 32) as u8);
        j += 1;
    }
    tmp = sum as i64;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as i16;
    pcm = pcm.add(1);
    //-- last 7 --
    coef = wincoef.add(255); /* back pass through coefs */
    i = 0;
    while i < 7 {
        coef = coef.sub(16);
        si = ((si as i32 - 1) as u8);
        bx = ((bx as i32 + 1) as u8);
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = ((si as i32 + 32) as u8);
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = ((bx as i32 + 32) as u8);
            j += 1;
        }
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(1);
        i += 1;
    }
}
//  --------------- 16 pt dual window (interleaved output) ------------------
pub unsafe fn window16_dual(vbuf: *const f32, mut vb_ptr: i32, mut pcm: *mut i16) {
    let mut i: i32;
    let mut j: i32;
    let mut si: u8;
    let mut bx: u8;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: i64;

    si = (vb_ptr + 8) as u8;
    bx = (si as i32 + 16) as u8;
    coef = wincoef;

    //-- first 8 --
    i = 0;
    while i < 8 {
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.add(1);
            si = ((si as i32 + 32) as u8);
            sum -= (*coef) * (*vbuf.add(bx as usize));
            coef = coef.add(1);
            bx = ((bx as i32 + 32) as u8);
            j += 1;
        }
        si = ((si as i32 + 1) as u8);
        bx = ((bx as i32 - 1) as u8);
        coef = coef.add(16);
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(2);
        i += 1;
    }
    //--  special case --
    sum = 0.0;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add(bx as usize));
        coef = coef.add(1);
        bx = ((bx as i32 + 32) as u8);
        j += 1;
    }
    tmp = sum as i64;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as i16;
    pcm = pcm.add(2);
    //-- last 7 --
    coef = wincoef.add(255); /* back pass through coefs */
    i = 0;
    while i < 7 {
        coef = coef.sub(16);
        si = ((si as i32 - 1) as u8);
        bx = ((bx as i32 + 1) as u8);
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add(si as usize));
            coef = coef.sub(1);
            si = ((si as i32 + 32) as u8);
            sum += (*coef) * (*vbuf.add(bx as usize));
            coef = coef.sub(1);
            bx = ((bx as i32 + 32) as u8);
            j += 1;
        }
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(2);
        i += 1;
    }
}
//  ------------------- 8 pt window --------------------------------
pub unsafe fn window8(vbuf: *const f32, mut vb_ptr: i32, mut pcm: *mut i16) {
    let mut i: i32;
    let mut j: i32;
    let mut si: i32;
    let mut bx: i32;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: i64;

    si = vb_ptr + 4;
    bx = (si + 8) & 127;
    coef = wincoef;

    //-- first 4 --
    i = 0;
    while i < 4 {
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 127));
            coef = coef.add(1);
            si = (si + 16) & 127;
            sum -= (*coef) * (*vbuf.add((bx as usize) & 127));
            coef = coef.add(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        si += 1;
        bx -= 1;
        coef = coef.add(48);
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(1);
        i += 1;
    }
    //--  special case --
    sum = 0.0;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add((bx as usize) & 127));
        coef = coef.add(1);
        bx = (bx + 16) & 127;
        j += 1;
    }
    tmp = sum as i64;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as i16;
    pcm = pcm.add(1);
    //-- last 3 --
    coef = wincoef.add(255); /* back pass through coefs */
    i = 0;
    while i < 3 {
        coef = coef.sub(48);
        si -= 1;
        bx += 1;
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 127));
            coef = coef.sub(1);
            si = (si + 16) & 127;
            sum += (*coef) * (*vbuf.add((bx as usize) & 127));
            coef = coef.sub(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(1);
        i += 1;
    }
}
//  --------------- 8 pt dual window (interleaved output) ------------------
pub unsafe fn window8_dual(vbuf: *const f32, mut vb_ptr: i32, mut pcm: *mut i16) {
    let mut i: i32;
    let mut j: i32;
    let mut si: i32;
    let mut bx: i32;
    let mut coef: *const f32;
    let mut sum: f32;
    let mut tmp: i64;

    si = vb_ptr + 4;
    bx = (si + 8) & 127;
    coef = wincoef;

    //-- first 4 --
    i = 0;
    while i < 4 {
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 127));
            coef = coef.add(1);
            si = (si + 16) & 127;
            sum -= (*coef) * (*vbuf.add((bx as usize) & 127));
            coef = coef.add(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        si += 1;
        bx -= 1;
        coef = coef.add(48);
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(2);
        i += 1;
    }
    //--  special case --
    sum = 0.0;
    j = 0;
    while j < 8 {
        sum += (*coef) * (*vbuf.add((bx as usize) & 127));
        coef = coef.add(1);
        bx = (bx + 16) & 127;
        j += 1;
    }
    tmp = sum as i64;
    if tmp > 32767 {
        tmp = 32767;
    } else if tmp < -32768 {
        tmp = -32768;
    }
    *pcm = tmp as i16;
    pcm = pcm.add(2);
    //-- last 3 --
    coef = wincoef.add(255); /* back pass through coefs */
    i = 0;
    while i < 3 {
        coef = coef.sub(48);
        si -= 1;
        bx += 1;
        sum = 0.0;
        j = 0;
        while j < 8 {
            sum += (*coef) * (*vbuf.add((si as usize) & 127));
            coef = coef.sub(1);
            si = (si + 16) & 127;
            sum += (*coef) * (*vbuf.add((bx as usize) & 127));
            coef = coef.sub(1);
            bx = (bx + 16) & 127;
            j += 1;
        }
        tmp = sum as i64;
        if tmp > 32767 {
            tmp = 32767;
        } else if tmp < -32768 {
            tmp = -32768;
        }
        *pcm = tmp as i16;
        pcm = pcm.add(2);
        i += 1;
    }
}
//  ---------------------------------------------------------------
