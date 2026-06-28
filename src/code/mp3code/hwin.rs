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

	$Id: hwin.c,v 1.5 1999/10/19 07:13:08 elrod Exp $
____________________________________________________________________________*/

/****  hwin.c  ***************************************************

Layer III

hybrid window/filter

******************************************************************/

use core::ptr::addr_of_mut;

#[allow(non_camel_case_types)]
type ARRAY36 = [f32; 36];

/*-- windows by block type --*/
#[allow(non_upper_case_globals)]
static mut WIN: [[f32; 36]; 4] = [[0.0; 36]; 4];	// effectively a constant

// Minimal struct for pMP3Stream reference from mp3struct.h
#[repr(C)]
pub struct MP3Stream {
    pub band_limit_nsb: i32,
    // ... other fields from mp3struct.h (not included in this port)
}

extern "C" {
	static mut pMP3Stream: *mut MP3Stream;
	fn imdct18(f: *mut f32);	/* 18 point */
	fn imdct6_3(f: *mut f32);	/* 6 point */
}

/*====================================================================*/
pub fn hwin_init_addr() -> *mut ARRAY36 {
	unsafe { addr_of_mut!(WIN[0]) as *mut ARRAY36 }
}


/*====================================================================*/
pub fn hybrid(
	xin: *mut f32,
	xprev: *mut f32,
	y: *mut [f32; 32],
	btype: i32,
	nlong: i32,
	ntot: i32,
	nprev: i32,
) -> i32 {
	let mut i: i32;
	let mut j: i32;
	let mut x: *mut f32;
	let mut x0: *mut f32;
	let mut xa: f32;
	let mut xb: f32;
	let mut n: i32;
	let mut nout: i32;

	let mut btype = btype;

	if btype == 2 {
		btype = 0;
	}
	x = xin;
	x0 = xprev;

	/*-- do long blocks (if any) --*/
	n = (nlong + 17) / 18;	/* number of dct's to do */
	i = 0;
	while i < n {
		unsafe {
			imdct18(x);
			j = 0;
			while j < 9 {
				(*y.add(j as usize))[i as usize] = *x0.add(j as usize)
					+ WIN[btype as usize][j as usize] * *x.add((9 + j) as usize);
				(*y.add((9 + j) as usize))[i as usize] = *x0.add((9 + j) as usize)
					+ WIN[btype as usize][(9 + j) as usize] * *x.add((17 - j) as usize);
				j += 1;
			}
			/*-- window x for next time x0 --*/
			j = 0;
			while j < 4 {
				xa = *x.add(j as usize);
				xb = *x.add((8 - j) as usize);
				*x.add(j as usize) = WIN[btype as usize][(18 + j) as usize] * xb;
				*x.add((8 - j) as usize) =
					WIN[btype as usize][((18 + 8) - j) as usize] * xa;
				*x.add((9 + j) as usize) =
					WIN[btype as usize][((18 + 9) + j) as usize] * xa;
				*x.add((17 - j) as usize) =
					WIN[btype as usize][((18 + 17) - j) as usize] * xb;
				j += 1;
			}
			xa = *x.add(j as usize);
			*x.add(j as usize) = WIN[btype as usize][(18 + j) as usize] * xa;
			*x.add((9 + j) as usize) =
				WIN[btype as usize][((18 + 9) + j) as usize] * xa;

			x = x.add(18);
			x0 = x0.add(18);
		}
		i += 1;
	}

	/*-- do short blocks (if any) --*/
	n = (ntot + 17) / 18;	/* number of 6 pt dct's triples to do */
	while i < n {
		unsafe {
			imdct6_3(x);
			j = 0;
			while j < 3 {
				(*y.add(j as usize))[i as usize] = *x0.add(j as usize);
				(*y.add((3 + j) as usize))[i as usize] = *x0.add((3 + j) as usize);

				(*y.add((6 + j) as usize))[i as usize] = *x0.add((6 + j) as usize)
					+ WIN[2][(j) as usize] * *x.add((3 + j) as usize);
				(*y.add((9 + j) as usize))[i as usize] = *x0.add((9 + j) as usize)
					+ WIN[2][(3 + j) as usize] * *x.add((5 - j) as usize);

				(*y.add((12 + j) as usize))[i as usize] = *x0.add((12 + j) as usize)
					+ WIN[2][(6 + j) as usize] * *x.add((2 - j) as usize)
					+ WIN[2][j as usize] * *x.add(((6 + 3) + j) as usize);
				(*y.add((15 + j) as usize))[i as usize] = *x0.add((15 + j) as usize)
					+ WIN[2][(9 + j) as usize] * *x.add(j as usize)
					+ WIN[2][(3 + j) as usize] * *x.add(((6 + 5) - j) as usize);
				j += 1;
			}
			/*-- window x for next time x0 --*/
			j = 0;
			while j < 3 {
				*x.add(j as usize) = WIN[2][(6 + j) as usize] * *x.add(((6 + 2) - j) as usize)
					+ WIN[2][j as usize] * *x.add(((12 + 3) + j) as usize);
				*x.add((3 + j) as usize) = WIN[2][(9 + j) as usize] * *x.add((6 + j) as usize)
					+ WIN[2][(3 + j) as usize] * *x.add(((12 + 5) - j) as usize);
				j += 1;
			}
			j = 0;
			while j < 3 {
				*x.add((6 + j) as usize) =
					WIN[2][(6 + j) as usize] * *x.add(((12 + 2) - j) as usize);
				*x.add((9 + j) as usize) = WIN[2][(9 + j) as usize] * *x.add((12 + j) as usize);
				j += 1;
			}
			j = 0;
			while j < 3 {
				*x.add((12 + j) as usize) = 0.0f32;
				*x.add((15 + j) as usize) = 0.0f32;
				j += 1;
			}
			x = x.add(18);
			x0 = x0.add(18);
		}
		i += 1;
	}

	/*--- overlap prev if prev longer that current --*/
	n = (nprev + 17) / 18;
	while i < n {
		unsafe {
			j = 0;
			while j < 18 {
				(*y.add(j as usize))[i as usize] = *x0.add(j as usize);
				j += 1;
			}
			x0 = x0.add(18);
		}
		i += 1;
	}
	nout = 18 * i;

	/*--- clear remaining only to band limit --*/
	unsafe {
		while i < (*pMP3Stream).band_limit_nsb {
			j = 0;
			while j < 18 {
				(*y.add(j as usize))[i as usize] = 0.0f32;
				j += 1;
			}
			i += 1;
		}
	}

	nout
}


/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
/*-- convert to mono, add curr result to y,
    window and add next time to current left */
pub fn hybrid_sum(
	xin: *mut f32,
	xin_left: *mut f32,
	y: *mut [f32; 32],
	btype: i32,
	nlong: i32,
	ntot: i32,
) -> i32 {
	let mut i: i32;
	let mut j: i32;
	let mut x: *mut f32;
	let mut x0: *mut f32;
	let mut xa: f32;
	let mut xb: f32;
	let mut n: i32;
	let mut nout: i32;

	let mut btype = btype;

	if btype == 2 {
		btype = 0;
	}
	x = xin;
	x0 = xin_left;

	/*-- do long blocks (if any) --*/
	n = (nlong + 17) / 18;	/* number of dct's to do */
	i = 0;
	while i < n {
		unsafe {
			imdct18(x);
			j = 0;
			while j < 9 {
				(*y.add(j as usize))[i as usize] += WIN[btype as usize][j as usize]
					* *x.add((9 + j) as usize);
				(*y.add((9 + j) as usize))[i as usize] +=
					WIN[btype as usize][(9 + j) as usize] * *x.add((17 - j) as usize);
				j += 1;
			}
			/*-- window x for next time x0 --*/
			j = 0;
			while j < 4 {
				xa = *x.add(j as usize);
				xb = *x.add((8 - j) as usize);
				*x0.add(j as usize) +=
					WIN[btype as usize][(18 + j) as usize] * xb;
				*x0.add((8 - j) as usize) +=
					WIN[btype as usize][((18 + 8) - j) as usize] * xa;
				*x0.add((9 + j) as usize) +=
					WIN[btype as usize][((18 + 9) + j) as usize] * xa;
				*x0.add((17 - j) as usize) +=
					WIN[btype as usize][((18 + 17) - j) as usize] * xb;
				j += 1;
			}
			xa = *x.add(j as usize);
			*x0.add(j as usize) += WIN[btype as usize][(18 + j) as usize] * xa;
			*x0.add((9 + j) as usize) +=
				WIN[btype as usize][((18 + 9) + j) as usize] * xa;

			x = x.add(18);
			x0 = x0.add(18);
		}
		i += 1;
	}

	/*-- do short blocks (if any) --*/
	n = (ntot + 17) / 18;	/* number of 6 pt dct's triples to do */
	while i < n {
		unsafe {
			imdct6_3(x);
			j = 0;
			while j < 3 {
				(*y.add((6 + j) as usize))[i as usize] +=
					WIN[2][j as usize] * *x.add((3 + j) as usize);
				(*y.add((9 + j) as usize))[i as usize] +=
					WIN[2][(3 + j) as usize] * *x.add((5 - j) as usize);

				(*y.add((12 + j) as usize))[i as usize] +=
					WIN[2][(6 + j) as usize] * *x.add((2 - j) as usize)
					+ WIN[2][j as usize] * *x.add(((6 + 3) + j) as usize);
				(*y.add((15 + j) as usize))[i as usize] +=
					WIN[2][(9 + j) as usize] * *x.add(j as usize)
					+ WIN[2][(3 + j) as usize] * *x.add(((6 + 5) - j) as usize);
				j += 1;
			}
			/*-- window x for next time --*/
			j = 0;
			while j < 3 {
				*x0.add(j as usize) += WIN[2][(6 + j) as usize] * *x.add(((6 + 2) - j) as usize)
					+ WIN[2][j as usize] * *x.add(((12 + 3) + j) as usize);
				*x0.add((3 + j) as usize) +=
					WIN[2][(9 + j) as usize] * *x.add((6 + j) as usize)
					+ WIN[2][(3 + j) as usize] * *x.add(((12 + 5) - j) as usize);
				j += 1;
			}
			j = 0;
			while j < 3 {
				*x0.add((6 + j) as usize) +=
					WIN[2][(6 + j) as usize] * *x.add(((12 + 2) - j) as usize);
				*x0.add((9 + j) as usize) += WIN[2][(9 + j) as usize] * *x.add((12 + j) as usize);
				j += 1;
			}
			x = x.add(18);
			x0 = x0.add(18);
		}
		i += 1;
	}

	nout = 18 * i;

	nout
}
/*--------------------------------------------------------------------*/
pub fn sum_f_bands(a: *mut f32, b: *const f32, n: i32) {
	let mut i: i32;

	i = 0;
	while i < n {
		unsafe {
			*a.add(i as usize) += *b.add(i as usize);
		}
		i += 1;
	}
}
/*--------------------------------------------------------------------*/
/*--------------------------------------------------------------------*/
#[allow(non_snake_case)]
pub fn FreqInvert(y: *mut [f32; 32], n: i32) {
	let mut i: i32;
	let mut j: i32;

	let n = (n + 17) / 18;
	j = 0;
	while j < 18 {
		if j % 2 == 0 {
			i = 0;
			while i < n {
				if i % 2 == 0 {
					unsafe {
						(*y.add((1 + j) as usize))[(1 + i) as usize] =
							-(*y.add((1 + j) as usize))[(1 + i) as usize];
					}
				}
				i += 1;
			}
		}
		j += 1;
	}
}
/*--------------------------------------------------------------------*/
