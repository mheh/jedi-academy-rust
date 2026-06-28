// pragma warning(disable:4206)	// nonstandard extension used : translation unit is empty
// #if 0
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
// Original C includes (disabled):
// #include <stdlib.h>
// #include <stdio.h>
// #include <float.h>
// #include <math.h>
// #ifdef WIN32
// #include <io.h>
// #else
// #include <unistd.h>
// #endif
// #include "port.h"

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BYTE_WAVE {
	pub riff: [u8; 4],
	pub size: [u8; 4],
	pub wave: [u8; 4],
	pub fmt: [u8; 4],
	pub fmtsize: [u8; 4],
	pub tag: [u8; 2],
	pub nChannels: [u8; 2],
	pub nSamplesPerSec: [u8; 4],
	pub nAvgBytesPerSec: [u8; 4],
	pub nBlockAlign: [u8; 2],
	pub nBitsPerSample: [u8; 2],
	pub data: [u8; 4],
	pub pcm_bytes: [u8; 4],
}

pub static wave: BYTE_WAVE = BYTE_WAVE {
	riff: [b'R', b'I', b'F', b'F'],
	size: [(std::mem::size_of::<BYTE_WAVE>() - 8) as u8, 0, 0, 0],
	wave: [b'W', b'A', b'V', b'E'],
	fmt: [b'f', b'm', b't', b' '],
	fmtsize: [16, 0, 0, 0],
	tag: [1, 0],
	nChannels: [1, 0],
	nSamplesPerSec: [34, 86, 0, 0],		/* 86 * 256 + 34 = 22050 */
	nAvgBytesPerSec: [172, 68, 0, 0],		/* 172 * 256 + 68 = 44100 */
	nBlockAlign: [2, 0],
	nBitsPerSample: [16, 0],
	data: [b'd', b'a', b't', b'a'],
	pcm_bytes: [0, 0, 0, 0],
};

/*----------------------------------------------------------------
  pcm conversion to wave format

  This conversion code required for big endian machine, or,
  if sizeof(short) != 16 bits.
  Conversion routines may be used on any machine, but if
  not required, the do nothing macros in port.h can be used instead
  to reduce overhead.

-----------------------------------------------------------------*/
// #ifndef LITTLE_SHORT16
// #include "wcvt.c"
// #endif
/*-----------------------------------------------*/
// #endif
