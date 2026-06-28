#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

// Original C starts with `#pragma warning(disable:4206)` for an empty
// translation unit, then wraps the entire body in `#if 0`.
#[cfg(any())]
mod wavep_c_fragment {
    use core::ffi::c_uchar;

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

        $Id: wavep.c,v 1.3 1999/10/19 07:13:09 elrod Exp $
    ____________________________________________________________________________*/

    /*---- wavep.c --------------------------------------------

    WAVE FILE HEADER ROUTINES
    with conditional pcm conversion to MS wave format
    portable version

    -----------------------------------------------------------*/

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct BYTE_WAVE {
        riff: [c_uchar; 4],
        size: [c_uchar; 4],
        wave: [c_uchar; 4],
        fmt: [c_uchar; 4],
        fmtsize: [c_uchar; 4],
        tag: [c_uchar; 2],
        nChannels: [c_uchar; 2],
        nSamplesPerSec: [c_uchar; 4],
        nAvgBytesPerSec: [c_uchar; 4],
        nBlockAlign: [c_uchar; 2],
        nBitsPerSample: [c_uchar; 2],
        data: [c_uchar; 4],
        pcm_bytes: [c_uchar; 4],
    }

    static wave: BYTE_WAVE = BYTE_WAVE {
        riff: *b"RIFF",
        size: [(core::mem::size_of::<BYTE_WAVE>() - 8) as c_uchar, 0, 0, 0],
        wave: *b"WAVE",
        fmt: *b"fmt ",
        fmtsize: [16, 0, 0, 0],
        tag: [1, 0],
        nChannels: [1, 0],
        nSamplesPerSec: [34, 86, 0, 0], /* 86 * 256 + 34 = 22050 */
        nAvgBytesPerSec: [172, 68, 0, 0], /* 172 * 256 + 68 = 44100 */
        nBlockAlign: [2, 0],
        nBitsPerSample: [16, 0],
        data: *b"data",
        pcm_bytes: [0, 0, 0, 0],
    };

    const _: () = assert!(core::mem::size_of::<BYTE_WAVE>() == 44);
    const _: () = assert!(core::mem::align_of::<BYTE_WAVE>() == 1);

    /*----------------------------------------------------------------
      pcm conversion to wave format

      This conversion code required for big endian machine, or,
      if sizeof(short) != 16 bits.
      Conversion routines may be used on any machine, but if
      not required, the do nothing macros in port.h can be used instead
      to reduce overhead.

    -----------------------------------------------------------------*/
    #[cfg(not(target_endian = "little"))]
    mod wcvt_c {
        use core::ffi::{c_int, c_uchar, c_uint};

        // Original C included "wcvt.c" here. That dependency is not present in
        // this port yet; keep the missing boundary explicit rather than
        // inventing replacement conversion behavior.
        unsafe extern "C" {
            pub fn cvt_to_wave_init(bits: c_int);
            pub fn cvt_to_wave(a: *mut c_uchar, b: c_uint) -> c_uint;
        }
    }
    /*-----------------------------------------------*/
}
