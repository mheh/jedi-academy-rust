/*
 * jdhuff.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains Huffman entropy decoding routines.
 *
 * Much of the complexity here has to do with supporting input suspension.
 * If the data source module demands suspension, we want to be able to back
 * up to the start of the current MCU.  To do this, we copy state variables
 * into local working storage, and update them back to the permanent
 * storage only upon successful completion of an MCU.
 */

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// leave this as first line for PCH reasons...
//
// exe_headers.h is a precompiled header — not ported as a standalone module.

use crate::code::jpeg_6::jinclude_h::*; /* MEMZERO, SIZEOF, etc. */
use crate::code::jpeg_6::jpeglib_h::*; /* jpeg_decompress_struct, JHUFF_TBL, jpeg_component_info,
                                         * jpeg_source_mgr, jpeg_memory_mgr, jpeg_error_mgr,
                                         * JBLOCKROW, JCOEF, JPOOL_IMAGE, NUM_HUFF_TBLS,
                                         * MAX_COMPS_IN_SCAN, DCTSIZE2, etc.
                                         */
/* jpeglib.h includes jpegint.h when JPEG_INTERNALS is defined.
 * Import jpeg_entropy_decoder from jpegint_h explicitly (overrides jpeglib_h dummy):
 */
use crate::code::jpeg_6::jpegint_h::jpeg_entropy_decoder;
/* jpeg_natural_order is a global from jutils.c, declared in jpegint.h: */
use crate::code::jpeg_6::jpegint_h::jpeg_natural_order;
use crate::code::jpeg_6::jdhuff_h::*; /* Declarations shared with jdphuff.c:
                                        * bitread_perm_state, bitread_working_state, d_derived_tbl,
                                        * bit_buf_type, HUFF_LOOKAHEAD, BIT_BUF_SIZE,
                                        * peek_bits, drop_bits, jpeg_fill_bit_buffer,
                                        * jpeg_huff_decode, jpeg_make_d_derived_tbl
                                        */
/* jpeglib.h includes jerror.h for error codes and macros: */
use crate::code::jpeg_6::jerror_h::J_MESSAGE_CODE;
/* FALSE/TRUE from jmorecfg.h (via jpeglib.h include chain): */
use crate::code::jpeg_6::jmorecfg_h::{FALSE, TRUE};

/* Explicit disambiguation: prefer jpeglib_h definitions for these names
 * over conflicting definitions in jdhuff_h / jpegint_h.
 */
use crate::code::jpeg_6::jpeglib_h::j_decompress_ptr;
use crate::code::jpeg_6::jpeglib_h::j_common_ptr;
use crate::code::jpeg_6::jpeglib_h::JHUFF_TBL;
use crate::code::jpeg_6::jpeglib_h::JBLOCKROW;

use core::ffi::{c_int, c_uint, c_void};
use core::ptr::addr_of_mut;

/* INT32: typedef long INT32 from jmorecfg.h.
 * Port note: jmorecfg_h.rs has this commented out; using i32 as proxy.
 */
type INT32 = i32;

/*
 * Expanded entropy decoder object for Huffman decoding.
 *
 * The savable_state subrecord contains fields that change within an MCU,
 * but must not be updated permanently until we complete the MCU.
 */

#[repr(C)]
struct savable_state {
    last_dc_val: [c_int; MAX_COMPS_IN_SCAN as usize], /* last DC coef for each component */
}

/* This macro is to work around compilers with missing or broken
 * structure assignment.  You'll need to fix this code if you have
 * such a compiler and you change MAX_COMPS_IN_SCAN.
 */

/* ASSIGN_STATE(dest,src): C macro ((dest) = (src)).
 * savable_state is Copy-assignable in Rust — plain assignment suffices.
 */
/* (No #else/#if MAX_COMPS_IN_SCAN == 4 alternative needed in C++.) */


#[repr(C)]
struct huff_entropy_decoder {
    pub_: jpeg_entropy_decoder, /* public fields */

    /* These fields are loaded into local variables at start of each MCU.
     * In case of suspension, we exit WITHOUT updating them.
     */
    bitstate: bitread_perm_state, /* Bit buffer at start of MCU */
    saved: savable_state,         /* Other state at start of MCU */

    /* These fields are NOT loaded into local working state. */
    restarts_to_go: c_uint, /* MCUs left in this restart interval */

    /* Pointers to derived tables (these workspaces have image lifespan) */
    dc_derived_tbls: [*mut d_derived_tbl; NUM_HUFF_TBLS as usize],
    ac_derived_tbls: [*mut d_derived_tbl; NUM_HUFF_TBLS as usize],
}

type huff_entropy_ptr = *mut huff_entropy_decoder;


/*
 * Initialize for a Huffman-compressed scan.
 */

unsafe extern "C" fn start_pass_huff_decoder(cinfo: j_decompress_ptr)
{
    let entropy: huff_entropy_ptr = (*cinfo).entropy as huff_entropy_ptr;
    let mut ci: c_int;
    let mut dctbl: c_int;
    let mut actbl: c_int;
    let mut compptr: *mut jpeg_component_info;

    /* Check that the scan parameters Ss, Se, Ah/Al are OK for sequential JPEG.
     * This ought to be an error condition, but we make it a warning because
     * there are some baseline files out there with all zeroes in these bytes.
     */
    if (*cinfo).Ss != 0 || (*cinfo).Se != DCTSIZE2 - 1 ||
       (*cinfo).Ah != 0 || (*cinfo).Al != 0 {
        /* WARNMS(cinfo, JWRN_NOT_SEQUENTIAL) — inline expansion: */
        (*(*cinfo).err).msg_code = J_MESSAGE_CODE::JWRN_NOT_SEQUENTIAL as c_int;
        if let Some(emit) = (*(*cinfo).err).emit_message {
            emit(cinfo as j_common_ptr, -1);
        }
    }

    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        dctbl = (*compptr).dc_tbl_no;
        actbl = (*compptr).ac_tbl_no;
        /* Make sure requested tables are present */
        if dctbl < 0 || dctbl >= NUM_HUFF_TBLS ||
           (*cinfo).dc_huff_tbl_ptrs[dctbl as usize].is_null() {
            /* ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, dctbl) — inline expansion: */
            (*(*cinfo).err).msg_code = J_MESSAGE_CODE::JERR_NO_HUFF_TABLE as c_int;
            (*(*cinfo).err).msg_parm.i[0] = dctbl;
            if let Some(exit_fn) = (*(*cinfo).err).error_exit {
                exit_fn(cinfo as j_common_ptr);
            }
        }
        if actbl < 0 || actbl >= NUM_HUFF_TBLS ||
           (*cinfo).ac_huff_tbl_ptrs[actbl as usize].is_null() {
            /* ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, actbl) — inline expansion: */
            (*(*cinfo).err).msg_code = J_MESSAGE_CODE::JERR_NO_HUFF_TABLE as c_int;
            (*(*cinfo).err).msg_parm.i[0] = actbl;
            if let Some(exit_fn) = (*(*cinfo).err).error_exit {
                exit_fn(cinfo as j_common_ptr);
            }
        }
        /* Compute derived values for Huffman tables */
        /* We may do this more than once for a table, but it's not expensive */
        jpeg_make_d_derived_tbl(cinfo as *mut c_void,
                                (*cinfo).dc_huff_tbl_ptrs[dctbl as usize] as *mut c_void,
                                addr_of_mut!((*entropy).dc_derived_tbls[dctbl as usize]));
        jpeg_make_d_derived_tbl(cinfo as *mut c_void,
                                (*cinfo).ac_huff_tbl_ptrs[actbl as usize] as *mut c_void,
                                addr_of_mut!((*entropy).ac_derived_tbls[actbl as usize]));
        /* Initialize DC predictions to 0 */
        (*entropy).saved.last_dc_val[ci as usize] = 0;
        ci += 1;
    }

    /* Initialize bitread state variables */
    (*entropy).bitstate.bits_left = 0;
    (*entropy).bitstate.get_buffer = 0; /* unnecessary, but keeps Purify quiet */
    (*entropy).bitstate.printed_eod = FALSE as c_int;

    /* Initialize restart counter */
    (*entropy).restarts_to_go = (*cinfo).restart_interval as c_uint;
}


/*
 * Compute the derived values for a Huffman table.
 * Note this is also used by jdphuff.c.
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_make_d_derived_tbl(cinfo: *mut c_void,
                                                  htbl: *mut c_void,
                                                  pdtbl: *mut *mut d_derived_tbl)
{
    // Port note: cinfo and htbl are `*mut c_void` to match the extern "C" decl in jdhuff_h.rs.
    // Cast to the concrete types before use.
    let cinfo = cinfo as j_decompress_ptr;
    let htbl = htbl as *mut JHUFF_TBL;
    let mut dtbl: *mut d_derived_tbl;
    let mut p: c_int;
    let mut i: c_int;
    let mut l: c_int;
    let mut si: c_int;
    let mut lookbits: c_int;
    let mut ctr: c_int;
    let mut huffsize: [c_int; 257] = [0; 257];
    let mut huffcode: [c_uint; 257] = [0; 257];
    let mut code: c_uint;

    /* Allocate a workspace if we haven't already done so. */
    if (*pdtbl).is_null() {
        *pdtbl = ((*(*cinfo).mem).alloc_small.unwrap())
            (cinfo as j_common_ptr, JPOOL_IMAGE, SIZEOF::<d_derived_tbl>())
            as *mut d_derived_tbl;
    }
    dtbl = *pdtbl;
    (*dtbl).pub_ = htbl; /* fill in back link */

    /* Figure C.1: make table of Huffman code length for each symbol */
    /* Note that this is in code-length order. */

    p = 0;
    l = 1;
    while l <= 16 {
        i = 1;
        while i <= (*htbl).bits[l as usize] as c_int {
            huffsize[p as usize] = l;
            p += 1;
            i += 1;
        }
        l += 1;
    }
    huffsize[p as usize] = 0;

    /* Figure C.2: generate the codes themselves */
    /* Note that this is in code-length order. */

    code = 0;
    si = huffsize[0];
    p = 0;
    while huffsize[p as usize] != 0 {
        while huffsize[p as usize] == si {
            huffcode[p as usize] = code;
            code = code.wrapping_add(1);
            p += 1;
        }
        code <<= 1;
        si += 1;
    }

    /* Figure F.15: generate decoding tables for bit-sequential decoding */

    p = 0;
    l = 1;
    while l <= 16 {
        if (*htbl).bits[l as usize] != 0 {
            (*dtbl).valptr[l as usize] = p; /* huffval[] index of 1st symbol of code length l */
            (*dtbl).mincode[l as usize] = huffcode[p as usize] as i32; /* minimum code of length l */
            p += (*htbl).bits[l as usize] as c_int;
            (*dtbl).maxcode[l as usize] = huffcode[(p - 1) as usize] as i32; /* maximum code of length l */
        } else {
            (*dtbl).maxcode[l as usize] = -1; /* -1 if no codes of this length */
        }
        l += 1;
    }
    (*dtbl).maxcode[17] = 0xFFFFFi32; /* ensures jpeg_huff_decode terminates */

    /* Compute lookahead tables to speed up decoding.
     * First we set all the table entries to 0, indicating "too long";
     * then we iterate through the Huffman codes that are short enough and
     * fill in all the entries that correspond to bit sequences starting
     * with that code.
     */

    /* MEMZERO(dtbl->look_nbits, SIZEOF(dtbl->look_nbits)) */
    MEMZERO(addr_of_mut!((*dtbl).look_nbits).cast::<c_void>(),
            SIZEOF::<[c_int; 256]>());

    p = 0;
    l = 1;
    while l <= HUFF_LOOKAHEAD {
        i = 1;
        while i <= (*htbl).bits[l as usize] as c_int {
            /* l = current code's length, p = its index in huffcode[] & huffval[]. */
            /* Generate left-justified code followed by all possible bit sequences */
            lookbits = (huffcode[p as usize] as c_int) << (HUFF_LOOKAHEAD - l);
            ctr = 1 << (HUFF_LOOKAHEAD - l);
            while ctr > 0 {
                (*dtbl).look_nbits[lookbits as usize] = l;
                (*dtbl).look_sym[lookbits as usize] = (*htbl).huffval[p as usize];
                lookbits += 1;
                ctr -= 1;
            }
            p += 1;
            i += 1;
        }
        l += 1;
    }
}


/*
 * Out-of-line code for bit fetching (shared with jdphuff.c).
 * See jdhuff.h for info about usage.
 * Note: current values of get_buffer and bits_left are passed as parameters,
 * but are returned in the corresponding fields of the state struct.
 *
 * On most machines MIN_GET_BITS should be 25 to allow the full 32-bit width
 * of get_buffer to be used.  (On machines with wider words, an even larger
 * buffer could be used.)  However, on some machines 32-bit shifts are
 * quite slow and take time proportional to the number of places shifted.
 * (This is true with most PC compilers, for instance.)  In this case it may
 * be a win to set MIN_GET_BITS to the minimum value of 15.  This reduces the
 * average shift distance at the cost of more calls to jpeg_fill_bit_buffer.
 */

/* #ifdef SLOW_SHIFT_32: MIN_GET_BITS = 15 (minimum allowable value) */
/* #else: */
const MIN_GET_BITS: c_int = BIT_BUF_SIZE - 7;
/* #endif */


#[no_mangle]
pub unsafe extern "C" fn jpeg_fill_bit_buffer(state: *mut bitread_working_state,
                                               mut get_buffer: bit_buf_type,
                                               mut bits_left: c_int,
                                               nbits: c_int)
    -> c_int
/* Load up the bit buffer to a depth of at least nbits */
{
    /* Copy heavily used state fields into locals (hopefully registers) */
    let mut next_input_byte: *const u8 = (*state).next_input_byte;
    let mut bytes_in_buffer: usize = (*state).bytes_in_buffer;
    let mut c: c_int;

    /* Port note: (*state).cinfo is *mut c_void (per jdhuff_h bitread_working_state).
     * Cast to j_decompress_ptr (= *mut jpeg_decompress_struct) for field access.
     */
    let state_cinfo = (*state).cinfo as j_decompress_ptr;

    /* Attempt to load at least MIN_GET_BITS bits into get_buffer. */
    /* (It is assumed that no request will be for more than that many bits.) */

    'fill: while bits_left < MIN_GET_BITS {
        /* Attempt to read a byte */

        /* Port note: C uses goto no_more_data from two sites in the loop body.
         * Restructured with a bool flag — control flow is semantically identical.
         */
        let mut no_more_data: bool = false;

        if (*state).unread_marker != 0 {
            no_more_data = true; /* can't advance past a marker */
        }

        c = 0;

        if !no_more_data {
            if bytes_in_buffer == 0 {
                if ((*(*state_cinfo).src).fill_input_buffer.unwrap())(state_cinfo) == 0 {
                    return FALSE as c_int;
                }
                next_input_byte = (*(*state_cinfo).src).next_input_byte;
                bytes_in_buffer = (*(*state_cinfo).src).bytes_in_buffer;
            }
            bytes_in_buffer -= 1;
            c = *next_input_byte as c_int; /* GETJOCTET(*next_input_byte++) */
            next_input_byte = next_input_byte.add(1);

            /* If it's 0xFF, check and discard stuffed zero byte */
            if c == 0xFF {
                loop {
                    if bytes_in_buffer == 0 {
                        if ((*(*state_cinfo).src).fill_input_buffer.unwrap())(state_cinfo) == 0 {
                            return FALSE as c_int;
                        }
                        next_input_byte = (*(*state_cinfo).src).next_input_byte;
                        bytes_in_buffer = (*(*state_cinfo).src).bytes_in_buffer;
                    }
                    bytes_in_buffer -= 1;
                    c = *next_input_byte as c_int; /* GETJOCTET(*next_input_byte++) */
                    next_input_byte = next_input_byte.add(1);
                    if c != 0xFF { break; }
                }

                if c == 0 {
                    /* Found FF/00, which represents an FF data byte */
                    c = 0xFF;
                } else {
                    /* Oops, it's actually a marker indicating end of compressed data. */
                    /* Better put it back for use later */
                    (*state).unread_marker = c;

                    no_more_data = true; /* goto no_more_data */
                }
            }
        }

        if no_more_data {
            /* no_more_data: */
            /* There should be enough bits still left in the data segment; */
            /* if so, just break out of the outer while loop. */
            if bits_left >= nbits {
                break 'fill;
            }
            /* Uh-oh.  Report corrupted data to user and stuff zeroes into
             * the data stream, so that we can produce some kind of image.
             * Note that this code will be repeated for each byte demanded
             * for the rest of the segment.  We use a nonvolatile flag to ensure
             * that only one warning message appears.
             */
            if *(*state).printed_eod_ptr == 0 {
                /* WARNMS(state->cinfo, JWRN_HIT_MARKER) — inline expansion: */
                (*(*state_cinfo).err).msg_code = J_MESSAGE_CODE::JWRN_HIT_MARKER as c_int;
                if let Some(emit) = (*(*state_cinfo).err).emit_message {
                    emit(state_cinfo as j_common_ptr, -1);
                }
                *(*state).printed_eod_ptr = TRUE as c_int;
            }
            c = 0; /* insert a zero byte into bit buffer */
        }

        /* OK, load c into get_buffer */
        get_buffer = (get_buffer << 8) | c;
        bits_left += 8;
    }

    /* Unload the local registers */
    (*state).next_input_byte = next_input_byte;
    (*state).bytes_in_buffer = bytes_in_buffer;
    (*state).get_buffer = get_buffer;
    (*state).bits_left = bits_left;

    return TRUE as c_int;
}


/*
 * Out-of-line code for Huffman code decoding.
 * See jdhuff.h for info about usage.
 */

#[no_mangle]
pub unsafe extern "C" fn jpeg_huff_decode(state: *mut bitread_working_state,
                                           mut get_buffer: bit_buf_type,
                                           mut bits_left: c_int,
                                           htbl: *mut d_derived_tbl,
                                           min_bits: c_int)
    -> c_int
{
    let mut l: c_int = min_bits;
    let mut code: INT32;

    /* HUFF_DECODE has determined that the code is at least min_bits */
    /* bits long, so fetch that many bits in one swoop. */

    /* CHECK_BIT_BUFFER(*state, l, return -1) — inline expansion: */
    if bits_left < l {
        if jpeg_fill_bit_buffer(state, get_buffer, bits_left, l) == 0 {
            return -1;
        }
        get_buffer = (*state).get_buffer;
        bits_left = (*state).bits_left;
    }
    /* GET_BITS(l) — inline expansion: */
    bits_left -= l;
    code = ((get_buffer >> bits_left) & ((1 << l) - 1)) as INT32;

    /* Collect the rest of the Huffman code one bit at a time. */
    /* This is per Figure F.16 in the JPEG spec. */

    while code > (*htbl).maxcode[l as usize] as INT32 {
        code <<= 1;
        /* CHECK_BIT_BUFFER(*state, 1, return -1) — inline expansion: */
        if bits_left < 1 {
            if jpeg_fill_bit_buffer(state, get_buffer, bits_left, 1) == 0 {
                return -1;
            }
            get_buffer = (*state).get_buffer;
            bits_left = (*state).bits_left;
        }
        /* GET_BITS(1) — inline expansion: */
        bits_left -= 1;
        code |= ((get_buffer >> bits_left) & 1) as INT32;
        l += 1;
    }

    /* Unload the local registers */
    (*state).get_buffer = get_buffer;
    (*state).bits_left = bits_left;

    /* With garbage input we may reach the sentinel value l = 17. */

    if l > 16 {
        /* WARNMS(state->cinfo, JWRN_HUFF_BAD_CODE) — inline expansion: */
        let state_cinfo = (*state).cinfo as j_decompress_ptr;
        (*(*state_cinfo).err).msg_code = J_MESSAGE_CODE::JWRN_HUFF_BAD_CODE as c_int;
        if let Some(emit) = (*(*state_cinfo).err).emit_message {
            emit(state_cinfo as j_common_ptr, -1);
        }
        return 0; /* fake a zero as the safest result */
    }

    return (*(*htbl).pub_).huffval[ ((*htbl).valptr[l as usize] +
                                    ((code - (*htbl).mincode[l as usize] as INT32) as c_int))
                                   as usize ] as c_int;
}


/*
 * Figure F.12: extend sign bit.
 * On some machines, a shift and add will be faster than a table lookup.
 */

/* #ifdef AVOID_TABLES:
 * #define HUFF_EXTEND(x,s)  ((x) < (1<<((s)-1)) ? (x) + (((-1)<<(s)) + 1) : (x))
 * #else: */

static extend_test: [c_int; 16] = /* entry n is 2**(n-1) */
  [ 0, 0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080,
    0x0100, 0x0200, 0x0400, 0x0800, 0x1000, 0x2000, 0x4000 ];

static extend_offset: [c_int; 16] = /* entry n is (-1 << n) + 1 */
  [ 0, ((-1i32)<<1) + 1, ((-1i32)<<2) + 1, ((-1i32)<<3) + 1, ((-1i32)<<4) + 1,
    ((-1i32)<<5) + 1, ((-1i32)<<6) + 1, ((-1i32)<<7) + 1, ((-1i32)<<8) + 1,
    ((-1i32)<<9) + 1, ((-1i32)<<10) + 1, ((-1i32)<<11) + 1, ((-1i32)<<12) + 1,
    ((-1i32)<<13) + 1, ((-1i32)<<14) + 1, ((-1i32)<<15) + 1 ];

/* #endif AVOID_TABLES */

/* HUFF_EXTEND(x,s): C macro translated as an inline function.
 * Uses the table form (non-AVOID_TABLES path).
 */
#[inline]
unsafe fn HUFF_EXTEND(x: c_int, s: c_int) -> c_int {
    if x < extend_test[s as usize] { x + extend_offset[s as usize] } else { x }
}


/*
 * Check for a restart marker & resynchronize decoder.
 * Returns FALSE if must suspend.
 */

unsafe fn process_restart(cinfo: j_decompress_ptr) -> c_int
{
    let entropy: huff_entropy_ptr = (*cinfo).entropy as huff_entropy_ptr;
    let mut ci: c_int;

    /* Port note: (*cinfo).marker is *mut jpeg_marker_reader (jpeglib_h dummy).
     * Cast to the full jpegint_h::jpeg_marker_reader to access discarded_bytes
     * and read_restart_marker fields.
     */
    let marker = (*cinfo).marker as *mut crate::code::jpeg_6::jpegint_h::jpeg_marker_reader;

    /* Throw away any unused bits remaining in bit buffer; */
    /* include any full bytes in next_marker's count of discarded bytes */
    (*marker).discarded_bytes =
        (*marker).discarded_bytes
            .wrapping_add(((*entropy).bitstate.bits_left / 8) as c_uint);
    (*entropy).bitstate.bits_left = 0;

    /* Advance past the RSTn marker */
    if ((*marker).read_restart_marker)(cinfo as *mut c_void) == 0 {
        return FALSE as c_int;
    }

    /* Re-initialize DC predictions to 0 */
    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        (*entropy).saved.last_dc_val[ci as usize] = 0;
        ci += 1;
    }

    /* Reset restart counter */
    (*entropy).restarts_to_go = (*cinfo).restart_interval as c_uint;

    /* Next segment can get another out-of-data warning */
    (*entropy).bitstate.printed_eod = FALSE as c_int;

    return TRUE as c_int;
}


/*
 * Decode and return one MCU's worth of Huffman-compressed coefficients.
 * The coefficients are reordered from zigzag order into natural array order,
 * but are not dequantized.
 *
 * The i'th block of the MCU is stored into the block pointed to by
 * MCU_data[i].  WE ASSUME THIS AREA HAS BEEN ZEROED BY THE CALLER.
 * (Wholesale zeroing is usually a little faster than retail...)
 *
 * Returns FALSE if data source requested suspension.  In that case no
 * changes have been made to permanent state.  (Exception: some output
 * coefficients may already have been assigned.  This is harmless for
 * this module, since we'll just re-assign them on the next call.)
 */

unsafe extern "C" fn decode_mcu(cinfo: j_decompress_ptr,
                                 MCU_data: *mut JBLOCKROW) -> c_int
{
    let entropy: huff_entropy_ptr = (*cinfo).entropy as huff_entropy_ptr;
    let mut s: c_int;
    let mut k: c_int;
    let mut r: c_int;
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut block: JBLOCKROW;
    /* BITREAD_STATE_VARS: */
    let mut get_buffer: bit_buf_type;
    let mut bits_left: c_int;
    let mut br_state: bitread_working_state = core::mem::zeroed();
    let mut state: savable_state;
    let mut dctbl: *mut d_derived_tbl;
    let mut actbl: *mut d_derived_tbl;
    let mut compptr: *mut jpeg_component_info;

    /* Process restart marker if needed; may have to suspend */
    if (*cinfo).restart_interval != 0 {
        if (*entropy).restarts_to_go == 0 {
            if process_restart(cinfo) == 0 {
                return FALSE as c_int;
            }
        }
    }

    /* Load up working state */
    /* BITREAD_LOAD_STATE(cinfo, entropy->bitstate) — inline expansion: */
    br_state.cinfo = cinfo as *mut c_void;
    br_state.next_input_byte = (*(*cinfo).src).next_input_byte;
    br_state.bytes_in_buffer = (*(*cinfo).src).bytes_in_buffer;
    br_state.unread_marker = (*cinfo).unread_marker;
    get_buffer = (*entropy).bitstate.get_buffer;
    bits_left = (*entropy).bitstate.bits_left;
    br_state.printed_eod_ptr = addr_of_mut!((*entropy).bitstate.printed_eod);
    /* ASSIGN_STATE(state, entropy->saved) — inline expansion (Copy assign): */
    state = savable_state { last_dc_val: (*entropy).saved.last_dc_val };

    /* Outer loop handles each block in the MCU */

    blkn = 0;
    while blkn < (*cinfo).blocks_in_MCU {
        block = *MCU_data.add(blkn as usize);
        ci = (*cinfo).MCU_membership[blkn as usize];
        compptr = (*cinfo).cur_comp_info[ci as usize];
        dctbl = (*entropy).dc_derived_tbls[(*compptr).dc_tbl_no as usize];
        actbl = (*entropy).ac_derived_tbls[(*compptr).ac_tbl_no as usize];

        /* Decode a single block's worth of coefficients */

        /* Section F.2.2.1: decode the DC coefficient difference */
        /* HUFF_DECODE(s, br_state, dctbl, return FALSE, label1) — goto-free expansion: */
        s = {
            if bits_left < HUFF_LOOKAHEAD {
                if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) == 0 {
                    return FALSE as c_int;
                }
                get_buffer = br_state.get_buffer;
                bits_left = br_state.bits_left;
                if bits_left < HUFF_LOOKAHEAD {
                    /* nb = 1; goto label1 (slow path) */
                    let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, dctbl, 1);
                    if tmp < 0 { return FALSE as c_int; }
                    get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                    tmp
                } else {
                    let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                    let nb = (*dctbl).look_nbits[look];
                    if nb != 0 { bits_left -= nb; (*dctbl).look_sym[look] as c_int }
                    else {
                        let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, dctbl, HUFF_LOOKAHEAD + 1);
                        if tmp < 0 { return FALSE as c_int; }
                        get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                        tmp
                    }
                }
            } else {
                let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                let nb = (*dctbl).look_nbits[look];
                if nb != 0 { bits_left -= nb; (*dctbl).look_sym[look] as c_int }
                else {
                    let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, dctbl, HUFF_LOOKAHEAD + 1);
                    if tmp < 0 { return FALSE as c_int; }
                    get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                    tmp
                }
            }
        };
        if s != 0 {
            /* CHECK_BIT_BUFFER(br_state, s, return FALSE) — inline expansion: */
            if bits_left < s {
                if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) == 0 {
                    return FALSE as c_int;
                }
                get_buffer = br_state.get_buffer;
                bits_left = br_state.bits_left;
            }
            /* r = GET_BITS(s) — inline expansion: */
            bits_left -= s;
            r = ((get_buffer >> bits_left) & ((1 << s) - 1)) as c_int;
            s = HUFF_EXTEND(r, s);
        }

        /* Shortcut if component's values are not interesting */
        if (*compptr).component_needed == 0 {
            /* goto skip_ACs */

            /* Section F.2.2.2: decode the AC coefficients */
            /* In this path we just discard the values */
            k = 1;
            while k < DCTSIZE2 {
                /* HUFF_DECODE(s, br_state, actbl, return FALSE, label3) — goto-free expansion: */
                s = {
                    if bits_left < HUFF_LOOKAHEAD {
                        if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) == 0 {
                            return FALSE as c_int;
                        }
                        get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                        if bits_left < HUFF_LOOKAHEAD {
                            let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, 1);
                            if tmp < 0 { return FALSE as c_int; }
                            get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                            tmp
                        } else {
                            let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                            let nb = (*actbl).look_nbits[look];
                            if nb != 0 { bits_left -= nb; (*actbl).look_sym[look] as c_int }
                            else {
                                let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD + 1);
                                if tmp < 0 { return FALSE as c_int; }
                                get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                tmp
                            }
                        }
                    } else {
                        let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                        let nb = (*actbl).look_nbits[look];
                        if nb != 0 { bits_left -= nb; (*actbl).look_sym[look] as c_int }
                        else {
                            let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD + 1);
                            if tmp < 0 { return FALSE as c_int; }
                            get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                            tmp
                        }
                    }
                };

                r = s >> 4;
                s &= 15;

                if s != 0 {
                    k += r;
                    /* CHECK_BIT_BUFFER(br_state, s, return FALSE) — inline expansion: */
                    if bits_left < s {
                        if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) == 0 {
                            return FALSE as c_int;
                        }
                        get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                    }
                    /* DROP_BITS(s) — inline expansion: */
                    bits_left -= s;
                } else {
                    if r != 15 {
                        break;
                    }
                    k += 15;
                }
                k += 1;
            }

        } else {

            /* Convert DC difference to actual value, update last_dc_val */
            s += state.last_dc_val[ci as usize];
            state.last_dc_val[ci as usize] = s;
            /* Output the DC coefficient (assumes jpeg_natural_order[0] = 0) */
            (*block)[0] = s as JCOEF;

            /* Do we need to decode the AC coefficients for this component? */
            if (*compptr).DCT_scaled_size > 1 {

                /* Section F.2.2.2: decode the AC coefficients */
                /* Since zeroes are skipped, output area must be cleared beforehand */
                k = 1;
                while k < DCTSIZE2 {
                    /* HUFF_DECODE(s, br_state, actbl, return FALSE, label2) — goto-free expansion: */
                    s = {
                        if bits_left < HUFF_LOOKAHEAD {
                            if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) == 0 {
                                return FALSE as c_int;
                            }
                            get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                            if bits_left < HUFF_LOOKAHEAD {
                                let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, 1);
                                if tmp < 0 { return FALSE as c_int; }
                                get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                tmp
                            } else {
                                let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                                let nb = (*actbl).look_nbits[look];
                                if nb != 0 { bits_left -= nb; (*actbl).look_sym[look] as c_int }
                                else {
                                    let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD + 1);
                                    if tmp < 0 { return FALSE as c_int; }
                                    get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                    tmp
                                }
                            }
                        } else {
                            let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                            let nb = (*actbl).look_nbits[look];
                            if nb != 0 { bits_left -= nb; (*actbl).look_sym[look] as c_int }
                            else {
                                let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD + 1);
                                if tmp < 0 { return FALSE as c_int; }
                                get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                tmp
                            }
                        }
                    };

                    r = s >> 4;
                    s &= 15;

                    if s != 0 {
                        k += r;
                        /* CHECK_BIT_BUFFER(br_state, s, return FALSE) — inline expansion: */
                        if bits_left < s {
                            if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) == 0 {
                                return FALSE as c_int;
                            }
                            get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                        }
                        /* r = GET_BITS(s) — inline expansion: */
                        bits_left -= s;
                        r = ((get_buffer >> bits_left) & ((1 << s) - 1)) as c_int;
                        s = HUFF_EXTEND(r, s);
                        /* Output coefficient in natural (dezigzagged) order.
                         * Note: the extra entries in jpeg_natural_order[] will save us
                         * if k >= DCTSIZE2, which could happen if the data is corrupted.
                         */
                        (*block)[jpeg_natural_order[k as usize] as usize] = s as JCOEF;
                    } else {
                        if r != 15 {
                            break;
                        }
                        k += 15;
                    }
                    k += 1;
                }

            } else {
                /* skip_ACs: */

                /* Section F.2.2.2: decode the AC coefficients */
                /* In this path we just discard the values */
                k = 1;
                while k < DCTSIZE2 {
                    /* HUFF_DECODE(s, br_state, actbl, return FALSE, label3) — goto-free expansion: */
                    s = {
                        if bits_left < HUFF_LOOKAHEAD {
                            if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) == 0 {
                                return FALSE as c_int;
                            }
                            get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                            if bits_left < HUFF_LOOKAHEAD {
                                let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, 1);
                                if tmp < 0 { return FALSE as c_int; }
                                get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                tmp
                            } else {
                                let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                                let nb = (*actbl).look_nbits[look];
                                if nb != 0 { bits_left -= nb; (*actbl).look_sym[look] as c_int }
                                else {
                                    let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD + 1);
                                    if tmp < 0 { return FALSE as c_int; }
                                    get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                    tmp
                                }
                            }
                        } else {
                            let look = ((get_buffer >> (bits_left - HUFF_LOOKAHEAD)) & ((1 << HUFF_LOOKAHEAD) - 1)) as usize;
                            let nb = (*actbl).look_nbits[look];
                            if nb != 0 { bits_left -= nb; (*actbl).look_sym[look] as c_int }
                            else {
                                let tmp = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD + 1);
                                if tmp < 0 { return FALSE as c_int; }
                                get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                                tmp
                            }
                        }
                    };

                    r = s >> 4;
                    s &= 15;

                    if s != 0 {
                        k += r;
                        /* CHECK_BIT_BUFFER(br_state, s, return FALSE) — inline expansion: */
                        if bits_left < s {
                            if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) == 0 {
                                return FALSE as c_int;
                            }
                            get_buffer = br_state.get_buffer; bits_left = br_state.bits_left;
                        }
                        /* DROP_BITS(s) — inline expansion: */
                        bits_left -= s;
                    } else {
                        if r != 15 {
                            break;
                        }
                        k += 15;
                    }
                    k += 1;
                }

            }
        }

        blkn += 1;
    }

    /* Completed MCU, so update state */
    /* BITREAD_SAVE_STATE(cinfo, entropy->bitstate) — inline expansion: */
    (*(*cinfo).src).next_input_byte = br_state.next_input_byte;
    (*(*cinfo).src).bytes_in_buffer = br_state.bytes_in_buffer;
    (*cinfo).unread_marker = br_state.unread_marker;
    (*entropy).bitstate.get_buffer = get_buffer;
    (*entropy).bitstate.bits_left = bits_left;
    /* ASSIGN_STATE(entropy->saved, state) — inline expansion (Copy assign): */
    (*entropy).saved.last_dc_val = state.last_dc_val;

    /* Account for restart interval (no-op if not using restarts) */
    (*entropy).restarts_to_go = (*entropy).restarts_to_go.wrapping_sub(1);

    return TRUE as c_int;
}


/*
 * Module initialization routine for Huffman entropy decoding.
 */

#[no_mangle]
pub unsafe extern "C" fn jinit_huff_decoder(cinfo: j_decompress_ptr)
{
    let entropy: huff_entropy_ptr;
    let mut i: c_int;

    entropy = ((*(*cinfo).mem).alloc_small.unwrap())
        (cinfo as j_common_ptr, JPOOL_IMAGE, SIZEOF::<huff_entropy_decoder>())
        as huff_entropy_ptr;
    /* cinfo->entropy = (struct jpeg_entropy_decoder *) entropy
     * Port note: (*cinfo).entropy is *mut jpeglib_h::jpeg_entropy_decoder (dummy struct).
     * Our huff_entropy_decoder begins with a jpegint_h::jpeg_entropy_decoder pub_ field.
     * Cast through *mut c_void to satisfy pointer type check.
     */
    (*cinfo).entropy = entropy as *mut c_void as *mut _;
    /* Port note: start_pass and decode_mcu use j_decompress_ptr = *mut jpeg_decompress_struct
     * but the pub_ field (jpegint_h::jpeg_entropy_decoder) expects fn(*mut c_void).
     * transmute bridges the incompatible fn pointer types.
     */
    (*entropy).pub_.start_pass = Some(core::mem::transmute::<
        unsafe extern "C" fn(j_decompress_ptr),
        extern "C" fn(*mut c_void),
    >(start_pass_huff_decoder));
    (*entropy).pub_.decode_mcu = Some(core::mem::transmute::<
        unsafe extern "C" fn(j_decompress_ptr, *mut JBLOCKROW) -> c_int,
        extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int,
    >(decode_mcu));

    /* Mark tables unallocated */
    i = 0;
    while i < NUM_HUFF_TBLS {
        (*entropy).dc_derived_tbls[i as usize] = core::ptr::null_mut();
        (*entropy).ac_derived_tbls[i as usize] = core::ptr::null_mut();
        i += 1;
    }
}
