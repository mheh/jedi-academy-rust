/* GLOBAL.H - RSAREF types and constants */
//Anything above this #include will be ignored by the compiler

use core::ffi::{c_int, c_void};

/* POINTER defines a generic pointer type */
type POINTER = *mut u8;

/* UINT2 defines a two byte word */
type UINT2 = u16;

/* UINT4 defines a four byte word */
type UINT4 = u32;


/* MD4.H - header file for MD4C.C */

/* Copyright (C) 1991-2, RSA Data Security, Inc. Created 1991.

All rights reserved.

License to copy and use this software is granted provided that it is identified as the "RSA Data Security, Inc. MD4 Message-Digest Algorithm" in all material mentioning or referencing this software or this function.
License is also granted to make and use derivative works provided that such works are identified as "derived from the RSA Data Security, Inc. MD4 Message-Digest Algorithm" in all material mentioning or referencing the derived work.
RSA Data Security, Inc. makes no representations concerning either the merchantability of this software or the suitability of this software for any particular purpose. It is provided "as is" without express or implied warranty of any kind.

These notices must be retained in any copies of any part of this documentation and/or software. */

/* MD4 context. */
#[repr(C)]
pub struct MD4_CTX {
	pub state: [UINT4; 4],				/* state (ABCD) */
	pub count: [UINT4; 2],				/* number of bits, modulo 2^64 (lsb first) */
	pub buffer: [u8; 64], 			/* input buffer */
}

extern "C" {
	fn Com_Memset (dest: *mut c_void, val: c_int, count: usize);
	fn Com_Memcpy (dest: *mut c_void, src: *const c_void, count: usize);
}


/* MD4C.C - RSA Data Security, Inc., MD4 message-digest algorithm */
/* Copyright (C) 1990-2, RSA Data Security, Inc. All rights reserved.

License to copy and use this software is granted provided that it is identified as the
RSA Data Security, Inc. MD4 Message-Digest Algorithm
 in all material mentioning or referencing this software or this function.
License is also granted to make and use derivative works provided that such works are identified as
derived from the RSA Data Security, Inc. MD4 Message-Digest Algorithm
in all material mentioning or referencing the derived work.
RSA Data Security, Inc. makes no representations concerning either the merchantability of this software or the suitability of this software for any particular purpose. It is provided
as is without express or implied warranty of any kind.

These notices must be retained in any copies of any part of this documentation and/or software. */

/* Constants for MD4Transform routine.  */
const S11: u32 = 3;
const S12: u32 = 7;
const S13: u32 = 11;
const S14: u32 = 19;
const S21: u32 = 3;
const S22: u32 = 5;
const S23: u32 = 9;
const S24: u32 = 13;
const S31: u32 = 3;
const S32: u32 = 9;
const S33: u32 = 11;
const S34: u32 = 15;

fn MD4Transform (state: &mut [UINT4; 4], block: &[u8; 64]);
fn Encode (output: *mut u8, input: *const UINT4, len: u32);
fn Decode (output: *mut UINT4, input: *const u8, len: u32);

static PADDING: [u8; 64] = [
0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

/* F, G and H are basic MD4 functions. */
#[inline]
fn F(x: UINT4, y: UINT4, z: UINT4) -> UINT4 {
	(((x) & (y)) | ((!(x)) & (z)))
}

#[inline]
fn G(x: UINT4, y: UINT4, z: UINT4) -> UINT4 {
	(((x) & (y)) | ((x) & (z)) | ((y) & (z)))
}

#[inline]
fn H(x: UINT4, y: UINT4, z: UINT4) -> UINT4 {
	((x) ^ (y) ^ (z))
}

/* ROTATE_LEFT rotates x left n bits. */
#[inline]
fn ROTATE_LEFT(x: UINT4, n: u32) -> UINT4 {
	(((x) << (n)) | ((x) >> (32-(n))))
}

/* FF, GG and HH are transformations for rounds 1, 2 and 3 */
/* Rotation is separate from addition to prevent recomputation */
macro_rules! FF {
	($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr) => {{
		$a = $a.wrapping_add(F($b, $c, $d)).wrapping_add($x);
		$a = ROTATE_LEFT($a, $s);
	}};
}

macro_rules! GG {
	($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr) => {{
		$a = $a.wrapping_add(G($b, $c, $d)).wrapping_add($x).wrapping_add(0x5a827999u32);
		$a = ROTATE_LEFT($a, $s);
	}};
}

macro_rules! HH {
	($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $s:expr) => {{
		$a = $a.wrapping_add(H($b, $c, $d)).wrapping_add($x).wrapping_add(0x6ed9eba1u32);
		$a = ROTATE_LEFT($a, $s);
	}};
}


/* MD4 initialization. Begins an MD4 operation, writing a new context. */
pub extern "C" fn MD4Init (context: *mut MD4_CTX)
{
	unsafe {
		(*context).count[0] = (*context).count[1] = 0;

	/* Load magic initialization constants.*/
	(*context).state[0] = 0x67452301;
	(*context).state[1] = 0xefcdab89;
	(*context).state[2] = 0x98badcfe;
	(*context).state[3] = 0x10325476;
	}
}

/* MD4 block update operation. Continues an MD4 message-digest operation, processing another message block, and updating the context. */
pub extern "C" fn MD4Update (context: *mut MD4_CTX, input: *const u8, inputLen: u32)
{
	let mut i: u32;
	let mut index: u32;
	let mut partLen: u32;

	unsafe {
		/* Compute number of bytes mod 64 */
		index = (((*context).count[0] >> 3) & 0x3F);

		/* Update number of bits */
		let shifted = inputLen << 3;
		let new_count = (*context).count[0].wrapping_add(shifted);
		if new_count < shifted {
			(*context).count[1] = (*context).count[1].wrapping_add(1);
		}
		(*context).count[0] = new_count;

		(*context).count[1] = (*context).count[1].wrapping_add((inputLen >> 29));

		partLen = 64 - index;

		/* Transform as many times as possible.*/
		if inputLen >= partLen
		{
	 		Com_Memcpy((&mut (*context).buffer[index as usize]) as *mut u8 as *mut c_void, input as *const c_void, partLen as usize);
	 		MD4Transform (&mut (*context).state, &(*context).buffer);

	 		i = partLen;
	 		while i + 63 < inputLen {
	 			MD4Transform (&mut (*context).state, &*(input.add(i as usize) as *const [u8; 64]));
	 			i = i.wrapping_add(64);
	 		}

	 		index = 0;
		}
		else {
	 		i = 0;
		}

		/* Buffer remaining input */
		Com_Memcpy ((&mut (*context).buffer[index as usize]) as *mut u8 as *mut c_void, (input.add(i as usize)) as *const c_void, (inputLen - i) as usize);
	}
}


/* MD4 finalization. Ends an MD4 message-digest operation, writing the the message digest and zeroizing the context. */
pub extern "C" fn MD4Final (digest: *mut u8, context: *mut MD4_CTX)
{
	let mut bits: [u8; 8] = [0; 8];
	let mut index: u32;
	let mut padLen: u32;

	unsafe {
		/* Save number of bits */
		Encode (bits.as_mut_ptr(), (*context).count.as_ptr(), 8);

		/* Pad out to 56 mod 64.*/
		index = (((*context).count[0] >> 3) & 0x3f);
		padLen = if index < 56 { 56 - index } else { 120 - index };
		MD4Update (context, PADDING.as_ptr(), padLen);

		/* Append length (before padding) */
		MD4Update (context, bits.as_ptr(), 8);

		/* Store state in digest */
		Encode (digest, (*context).state.as_ptr(), 16);

		/* Zeroize sensitive information.*/
		Com_Memset ((context as *mut u8) as *mut c_void, 0, core::mem::size_of::<MD4_CTX>());
	}
}


/* MD4 basic transformation. Transforms state based on block. */
fn MD4Transform (state: &mut [UINT4; 4], block: &[u8; 64])
{
	let mut a: UINT4 = state[0];
	let mut b: UINT4 = state[1];
	let mut c: UINT4 = state[2];
	let mut d: UINT4 = state[3];
	let mut x: [UINT4; 16] = [0; 16];

	Decode (x.as_mut_ptr(), block.as_ptr(), 64);

/* Round 1 */
FF! (a, b, c, d, x[ 0], S11); 				/* 1 */
FF! (d, a, b, c, x[ 1], S12); 				/* 2 */
FF! (c, d, a, b, x[ 2], S13); 				/* 3 */
FF! (b, c, d, a, x[ 3], S14); 				/* 4 */
FF! (a, b, c, d, x[ 4], S11); 				/* 5 */
FF! (d, a, b, c, x[ 5], S12); 				/* 6 */
FF! (c, d, a, b, x[ 6], S13); 				/* 7 */
FF! (b, c, d, a, x[ 7], S14); 				/* 8 */
FF! (a, b, c, d, x[ 8], S11); 				/* 9 */
FF! (d, a, b, c, x[ 9], S12); 				/* 10 */
FF! (c, d, a, b, x[10], S13); 			/* 11 */
FF! (b, c, d, a, x[11], S14); 			/* 12 */
FF! (a, b, c, d, x[12], S11); 			/* 13 */
FF! (d, a, b, c, x[13], S12); 			/* 14 */
FF! (c, d, a, b, x[14], S13); 			/* 15 */
FF! (b, c, d, a, x[15], S14); 			/* 16 */

/* Round 2 */
GG! (a, b, c, d, x[ 0], S21); 			/* 17 */
GG! (d, a, b, c, x[ 4], S22); 			/* 18 */
GG! (c, d, a, b, x[ 8], S23); 			/* 19 */
GG! (b, c, d, a, x[12], S24); 			/* 20 */
GG! (a, b, c, d, x[ 1], S21); 			/* 21 */
GG! (d, a, b, c, x[ 5], S22); 			/* 22 */
GG! (c, d, a, b, x[ 9], S23); 			/* 23 */
GG! (b, c, d, a, x[13], S24); 			/* 24 */
GG! (a, b, c, d, x[ 2], S21); 			/* 25 */
GG! (d, a, b, c, x[ 6], S22); 			/* 26 */
GG! (c, d, a, b, x[10], S23); 			/* 27 */
GG! (b, c, d, a, x[14], S24); 			/* 28 */
GG! (a, b, c, d, x[ 3], S21); 			/* 29 */
GG! (d, a, b, c, x[ 7], S22); 			/* 30 */
GG! (c, d, a, b, x[11], S23); 			/* 31 */
GG! (b, c, d, a, x[15], S24); 			/* 32 */

/* Round 3 */
HH! (a, b, c, d, x[ 0], S31);				/* 33 */
HH! (d, a, b, c, x[ 8], S32); 			/* 34 */
HH! (c, d, a, b, x[ 4], S33); 			/* 35 */
HH! (b, c, d, a, x[12], S34); 			/* 36 */
HH! (a, b, c, d, x[ 2], S31); 			/* 37 */
HH! (d, a, b, c, x[10], S32); 			/* 38 */
HH! (c, d, a, b, x[ 6], S33); 			/* 39 */
HH! (b, c, d, a, x[14], S34); 			/* 40 */
HH! (a, b, c, d, x[ 1], S31); 			/* 41 */
HH! (d, a, b, c, x[ 9], S32); 			/* 42 */
HH! (c, d, a, b, x[ 5], S33); 			/* 43 */
HH! (b, c, d, a, x[13], S34); 			/* 44 */
HH! (a, b, c, d, x[ 3], S31); 			/* 45 */
HH! (d, a, b, c, x[11], S32); 			/* 46 */
HH! (c, d, a, b, x[ 7], S33); 			/* 47 */
HH! (b, c, d, a, x[15], S34);			/* 48 */

state[0] = state[0].wrapping_add(a);
state[1] = state[1].wrapping_add(b);
state[2] = state[2].wrapping_add(c);
state[3] = state[3].wrapping_add(d);

	/* Zeroize sensitive information.*/
	Com_Memset (x.as_mut_ptr() as *mut c_void, 0, core::mem::size_of_val(&x));
}


/* Encodes input (UINT4) into output (unsigned char). Assumes len is a multiple of 4. */
fn Encode (output: *mut u8, input: *const UINT4, len: u32)
{
	let mut i: u32 = 0;
	let mut j: u32 = 0;

	while j < len {
 		unsafe {
 			*output.add(j as usize) = ((*input.add(i as usize)) & 0xff) as u8;
 			*output.add((j+1) as usize) = (((*input.add(i as usize)) >> 8) & 0xff) as u8;
 			*output.add((j+2) as usize) = (((*input.add(i as usize)) >> 16) & 0xff) as u8;
 			*output.add((j+3) as usize) = (((*input.add(i as usize)) >> 24) & 0xff) as u8;
 		}
		i = i.wrapping_add(1);
		j = j.wrapping_add(4);
	}
}


/* Decodes input (unsigned char) into output (UINT4). Assumes len is a multiple of 4. */
fn Decode (output: *mut UINT4, input: *const u8, len: u32)
{
	let mut i: u32 = 0;
	let mut j: u32 = 0;

	while j < len {
 		unsafe {
 			*output.add(i as usize) = ((*input.add(j as usize)) as UINT4) | (((*input.add((j+1) as usize)) as UINT4) << 8) | (((*input.add((j+2) as usize)) as UINT4) << 16) | (((*input.add((j+3) as usize)) as UINT4) << 24);
 		}
		i = i.wrapping_add(1);
		j = j.wrapping_add(4);
	}
}

//===================================================================

pub extern "C" fn Com_BlockChecksum (buffer: *const c_void, length: c_int) -> u32
{
	let mut digest: [c_int; 4] = [0; 4];
	let mut val: u32;
	let mut ctx: MD4_CTX = unsafe { core::mem::zeroed() };

	MD4Init (&mut ctx);
	MD4Update (&mut ctx, buffer as *const u8, length as u32);
	MD4Final ( digest.as_mut_ptr() as *mut u8, &mut ctx);

	val = (digest[0] as u32) ^ (digest[1] as u32) ^ (digest[2] as u32) ^ (digest[3] as u32);

	return val;
}

pub extern "C" fn Com_BlockChecksumKey (buffer: *mut c_void, length: c_int, key: c_int) -> u32
{
	let mut digest: [c_int; 4] = [0; 4];
	let mut val: u32;
	let mut ctx: MD4_CTX = unsafe { core::mem::zeroed() };

	MD4Init (&mut ctx);
	MD4Update (&mut ctx, (&key as *const c_int) as *const u8, 4);
	MD4Update (&mut ctx, buffer as *const u8, length as u32);
	MD4Final ( digest.as_mut_ptr() as *mut u8, &mut ctx);

	val = (digest[0] as u32) ^ (digest[1] as u32) ^ (digest[2] as u32) ^ (digest[3] as u32);

	return val;
}
