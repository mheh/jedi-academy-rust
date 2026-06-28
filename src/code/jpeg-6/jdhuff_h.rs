/*
 * jdhuff.h
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains declarations for Huffman entropy decoding routines
 * that are shared between the sequential decoder (jdhuff.c) and the
 * progressive decoder (jdphuff.c).  No other modules need to see these.
 */

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};

/* Derived data constructed for each Huffman table */

pub const HUFF_LOOKAHEAD: c_int = 8; /* # of bits of lookahead */

#[repr(C)]
pub struct d_derived_tbl {
    /* Basic tables: (element [0] of each array is unused) */
    pub mincode: [i32; 17], /* smallest code of length k */
    pub maxcode: [i32; 18], /* largest code of length k (-1 if none) */
    /* (maxcode[17] is a sentinel to ensure jpeg_huff_decode terminates) */
    pub valptr: [c_int; 17], /* huffval[] index of 1st symbol of length k */

    /* Link to public Huffman table (needed only in jpeg_huff_decode) */
    pub pub_: *mut JHUFF_TBL,

    /* Lookahead tables: indexed by the next HUFF_LOOKAHEAD bits of
     * the input data stream.  If the next Huffman code is no more
     * than HUFF_LOOKAHEAD bits long, we can obtain its length and
     * the corresponding symbol directly from these tables.
     */
    pub look_nbits: [c_int; 256], /* # bits, or 0 if too long */
    pub look_sym: [u8; 256],       /* symbol, or unused */
}

/* Opaque type for JHUFF_TBL - defined in jpeglib.h */
#[repr(C)]
pub struct JHUFF_TBL {
    _private: [u8; 0],
}

pub type j_decompress_ptr = *mut c_void; /* opaque pointer to jpeg_decompress_struct */

/* Expand a Huffman table definition into the derived format */
extern "C" {
    pub fn jpeg_make_d_derived_tbl(
        cinfo: j_decompress_ptr,
        htbl: *mut JHUFF_TBL,
        pdtbl: *mut *mut d_derived_tbl,
    );
}

/*
 * Fetching the next N bits from the input stream is a time-critical operation
 * for the Huffman decoders.  We implement it with a combination of inline
 * macros and out-of-line subroutines.  Note that N (the number of bits
 * demanded at one time) never exceeds 15 for JPEG use.
 *
 * We read source bytes into get_buffer and dole out bits as needed.
 * If get_buffer already contains enough bits, they are fetched in-line
 * by the macros CHECK_BIT_BUFFER and GET_BITS.  When there aren't enough
 * bits, jpeg_fill_bit_buffer is called; it will attempt to fill get_buffer
 * as full as possible (not just to the number of bits needed; this
 * prefetching reduces the overhead cost of calling jpeg_fill_bit_buffer).
 * Note that jpeg_fill_bit_buffer may return FALSE to indicate suspension.
 * On TRUE return, jpeg_fill_bit_buffer guarantees that get_buffer contains
 * at least the requested number of bits --- dummy zeroes are inserted if
 * necessary.
 */

pub type bit_buf_type = i32; /* type of bit-extraction buffer */
pub const BIT_BUF_SIZE: c_int = 32; /* size of buffer in bits */

/* If long is > 32 bits on your machine, and shifting/masking longs is
 * reasonably fast, making bit_buf_type be long and setting BIT_BUF_SIZE
 * appropriately should be a win.  Unfortunately we can't do this with
 * something like  #define BIT_BUF_SIZE (sizeof(bit_buf_type)*8)
 * because not all machines measure sizeof in 8-bit bytes.
 */

#[repr(C)]
pub struct bitread_perm_state {
    /* Bitreading state saved across MCUs */
    pub get_buffer: bit_buf_type,  /* current bit-extraction buffer */
    pub bits_left: c_int,          /* # of unused bits in it */
    pub printed_eod: c_int,        /* flag to suppress multiple warning msgs */
}

#[repr(C)]
pub struct bitread_working_state {
    /* Bitreading working state within an MCU */
    /* current data source state */
    pub next_input_byte: *const u8, /* => next byte to read from source */
    pub bytes_in_buffer: usize,     /* # of bytes remaining in source buffer */
    pub unread_marker: c_int,       /* nonzero if we have hit a marker */
    /* bit input buffer --- note these values are kept in register variables,
     * not in this struct, inside the inner loops.
     */
    pub get_buffer: bit_buf_type, /* current bit-extraction buffer */
    pub bits_left: c_int,         /* # of unused bits in it */
    /* pointers needed by jpeg_fill_bit_buffer */
    pub cinfo: j_decompress_ptr, /* back link to decompress master record */
    pub printed_eod_ptr: *mut c_int, /* => flag in permanent state */
}

/* Macros to declare and load/save bitread local variables. */
/* BITREAD_STATE_VARS would declare:
 *   register bit_buf_type get_buffer;
 *   register int bits_left;
 *   bitread_working_state br_state
 * These are used in the actual Huffman decoder loops in C.
 * In Rust, we need to use explicit local variables and pass the state struct.
 */

/* BITREAD_LOAD_STATE(cinfop,permstate) would load:
 *   br_state.cinfo = cinfop;
 *   br_state.next_input_byte = cinfop->src->next_input_byte;
 *   br_state.bytes_in_buffer = cinfop->src->bytes_in_buffer;
 *   br_state.unread_marker = cinfop->unread_marker;
 *   get_buffer = permstate.get_buffer;
 *   bits_left = permstate.bits_left;
 *   br_state.printed_eod_ptr = & permstate.printed_eod
 */

/* BITREAD_SAVE_STATE(cinfop,permstate) would save:
 *   cinfop->src->next_input_byte = br_state.next_input_byte;
 *   cinfop->src->bytes_in_buffer = br_state.bytes_in_buffer;
 *   cinfop->unread_marker = br_state.unread_marker;
 *   permstate.get_buffer = get_buffer;
 *   permstate.bits_left = bits_left
 */

/*
 * These macros provide the in-line portion of bit fetching.
 * Use CHECK_BIT_BUFFER to ensure there are N bits in get_buffer
 * before using GET_BITS, PEEK_BITS, or DROP_BITS.
 * The variables get_buffer and bits_left are assumed to be locals,
 * but the state struct might not be (jpeg_huff_decode needs this).
 *	CHECK_BIT_BUFFER(state,n,action);
 *		Ensure there are N bits in get_buffer; if suspend, take action.
 *      val = GET_BITS(n);
 *		Fetch next N bits.
 *      val = PEEK_BITS(n);
 *		Fetch next N bits without removing them from the buffer.
 *	DROP_BITS(n);
 *		Discard next N bits.
 * The value N should be a simple variable, not an expression, because it
 * is evaluated multiple times.
 */

/* In C, these are macros. In Rust, we provide inline helper functions instead.
 * The macros would be:
 *
 * #define CHECK_BIT_BUFFER(state,nbits,action) \
 *     { if (bits_left < (nbits)) {  \
 *         if (! jpeg_fill_bit_buffer(&(state),get_buffer,bits_left,nbits))  \
 *           { action; }  \
 *         get_buffer = (state).get_buffer; bits_left = (state).bits_left; } }
 *
 * #define GET_BITS(nbits) \
 *     (((int) (get_buffer >> (bits_left -= (nbits)))) & ((1<<(nbits))-1))
 *
 * #define PEEK_BITS(nbits) \
 *     (((int) (get_buffer >> (bits_left -  (nbits)))) & ((1<<(nbits))-1))
 *
 * #define DROP_BITS(nbits) \
 *     (bits_left -= (nbits))
 */

/* Inline helper for GET_BITS semantics: fetch N bits and advance */
#[inline]
pub fn get_bits(get_buffer: bit_buf_type, bits_left: &mut c_int, nbits: c_int) -> c_int {
    *bits_left -= nbits;
    let val: c_int = ((get_buffer >> *bits_left) & ((1 << nbits) - 1)) as c_int;
    val
}

/* Inline helper for PEEK_BITS semantics: fetch N bits without advancing */
#[inline]
pub fn peek_bits(get_buffer: bit_buf_type, bits_left: c_int, nbits: c_int) -> c_int {
    let val: c_int =
        ((get_buffer >> (bits_left - nbits)) & ((1 << nbits) - 1)) as c_int;
    val
}

/* Inline helper for DROP_BITS semantics: discard N bits */
#[inline]
pub fn drop_bits(bits_left: &mut c_int, nbits: c_int) {
    *bits_left -= nbits;
}

/* Load up the bit buffer to a depth of at least nbits */
extern "C" {
    pub fn jpeg_fill_bit_buffer(
        state: *mut bitread_working_state,
        get_buffer: bit_buf_type,
        bits_left: c_int,
        nbits: c_int,
    ) -> c_int;
}

/*
 * Code for extracting next Huffman-coded symbol from input bit stream.
 * Again, this is time-critical and we make the main paths be macros.
 *
 * We use a lookahead table to process codes of up to HUFF_LOOKAHEAD bits
 * without looping.  Usually, more than 95% of the Huffman codes will be 8
 * or fewer bits long.  The few overlength codes are handled with a loop,
 * which need not be inline code.
 *
 * Notes about the HUFF_DECODE macro:
 * 1. Near the end of the data segment, we may fail to get enough bits
 *    for a lookahead.  In that case, we do it the hard way.
 * 2. If the lookahead table contains no entry, the next code must be
 *    more than HUFF_LOOKAHEAD bits long.
 * 3. jpeg_huff_decode returns -1 if forced to suspend.
 */

/* In C, this is a macro that involves gotos and register variables:
 *
 * #define HUFF_DECODE(result,state,htbl,failaction,slowlabel) \
 * { register int nb, look; \
 *   if (bits_left < HUFF_LOOKAHEAD) { \
 *     if (! jpeg_fill_bit_buffer(&state,get_buffer,bits_left, 0)) {failaction;} \
 *     get_buffer = state.get_buffer; bits_left = state.bits_left; \
 *     if (bits_left < HUFF_LOOKAHEAD) { \
 *       nb = 1; goto slowlabel; \
 *     } \
 *   } \
 *   look = PEEK_BITS(HUFF_LOOKAHEAD); \
 *   if ((nb = htbl->look_nbits[look]) != 0) { \
 *     DROP_BITS(nb); \
 *     result = htbl->look_sym[look]; \
 *   } else { \
 *     nb = HUFF_LOOKAHEAD+1; \
 * slowlabel: \
 *     if ((result=jpeg_huff_decode(&state,get_buffer,bits_left,htbl,nb)) < 0) \
 *         { failaction; } \
 *     get_buffer = state.get_buffer; bits_left = state.bits_left; \
 *   } \
 * }
 * This is too complex to translate to a Rust inline, so users will need to
 * adapt this logic in their decoder loops.
 */

/* Out-of-line case for Huffman code fetching */
extern "C" {
    pub fn jpeg_huff_decode(
        state: *mut bitread_working_state,
        get_buffer: bit_buf_type,
        bits_left: c_int,
        htbl: *mut d_derived_tbl,
        min_bits: c_int,
    ) -> c_int;
}
