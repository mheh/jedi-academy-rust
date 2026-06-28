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

use core::ffi::{c_int, c_void};
use core::mem::size_of;

// ============================================================================
// Stubs for JPEG-6 types and structures needed for structural coherence
// ============================================================================

pub type JDIMENSION = u32;
pub type JOCTET = u8;
pub type JSAMPROW = *mut u8;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JSAMPIMAGE = *mut JSAMPARRAY;
pub type boolean = u8;
pub type JCOEF = i16;
pub type INT32 = i32;
pub type UINT8 = u8;
pub type INT16 = i16;

// Constants from jpeglib.h
const DCTSIZE2: usize = 64;            /* DCTSIZE squared; # of elements in a block */
const NUM_HUFF_TBLS: usize = 4;        /* Huffman tables are numbered 0..3 */
const NUM_QUANT_TBLS: usize = 4;       /* Quantization tables are numbered 0..3 */
const MAX_COMPS_IN_SCAN: usize = 4;    /* JPEG limit on # of components in one scan */
const MAX_COMPONENTS: usize = 10;      /* maximum number of image components */

const FALSE: boolean = 0;
const TRUE: boolean = 1;

const JPOOL_IMAGE: c_int = 1;

const JERR_NO_HUFF_TABLE: c_int = 1;

const JWRN_NOT_SEQUENTIAL: c_int = 1;
const JWRN_HIT_MARKER: c_int = 2;
const JWRN_HUFF_BAD_CODE: c_int = 3;

// Huffman table lookahead size
const HUFF_LOOKAHEAD: usize = 8;       /* # of bits of lookahead */

// Bit buffer type and size
pub type bit_buf_type = INT32;
const BIT_BUF_SIZE: usize = 32;        /* size of buffer in bits */

#[cfg(target_pointer_width = "32")]
const MIN_GET_BITS: usize = BIT_BUF_SIZE - 7;

// Forward declarations of structures
#[repr(C)]
pub struct JQUANT_TBL {
    pub quantval: [u16; DCTSIZE2],
    pub sent_table: boolean,
}

#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [u8; 17],
    pub huffval: [u8; 256],
    pub sent_table: boolean,
}

#[repr(C)]
pub struct jpeg_error_mgr {
    pub msg_code: c_int,
    pub msg_parm: msg_parm_union,
    pub error_exit: Option<unsafe extern "C" fn(*mut c_void)>,
    pub emit_message: Option<unsafe extern "C" fn(*mut c_void, c_int)>,
}

#[repr(C)]
pub union msg_parm_union {
    pub i: [c_int; 8],
    pub s: [u8; 80],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
}

#[repr(C)]
pub struct jpeg_source_mgr {
    pub next_input_byte: *const JOCTET,
    pub bytes_in_buffer: usize,
    pub fill_input_buffer: Option<unsafe extern "C" fn(*mut c_void) -> boolean>,
}

#[repr(C)]
pub struct jpeg_marker_struct {
    pub discarded_bytes: usize,
    pub read_restart_marker: Option<unsafe extern "C" fn(*mut c_void) -> boolean>,
}

#[repr(C)]
pub struct jpeg_entropy_decoder {
    pub start_pass: Option<unsafe extern "C" fn(*mut c_void)>,
    pub decode_mcu: Option<unsafe extern "C" fn(*mut c_void, *mut *mut JCOEF) -> boolean>,
}

// Type aliases for function pointers used in entropy decoder
pub type start_pass_fn = unsafe fn(j_decompress_ptr);
pub type decode_mcu_fn = unsafe fn(j_decompress_ptr, *mut JBLOCKROW) -> boolean;

#[repr(C)]
pub struct jpeg_component_info {
    pub component_id: c_int,
    pub component_index: c_int,
    pub h_samp_factor: c_int,
    pub v_samp_factor: c_int,
    pub quant_tbl_no: c_int,
    pub dc_tbl_no: c_int,
    pub ac_tbl_no: c_int,
    pub width_in_blocks: JDIMENSION,
    pub height_in_blocks: JDIMENSION,
    pub DCT_scaled_size: c_int,
    pub downsampled_width: JDIMENSION,
    pub downsampled_height: JDIMENSION,
    pub component_needed: boolean,
    pub MCU_width: c_int,
    pub MCU_height: c_int,
    pub MCU_blocks: c_int,
    pub MCU_sample_width: c_int,
    pub last_col_width: c_int,
    pub last_row_height: c_int,
    pub quant_table: *mut JQUANT_TBL,
    pub dct_table: *mut c_void,
}

#[repr(C)]
pub struct j_decompress_struct {
    pub err: *mut jpeg_error_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub is_decompressor: boolean,
    pub global_state: c_int,
    pub src: *mut jpeg_source_mgr,
    pub image_width: JDIMENSION,
    pub image_height: JDIMENSION,
    pub num_components: c_int,
    pub jpeg_color_space: c_int,
    pub comp_info: *mut jpeg_component_info,
    pub quant_tbl_ptrs: [*mut JQUANT_TBL; NUM_QUANT_TBLS],
    pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; NUM_HUFF_TBLS],
    pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; NUM_HUFF_TBLS],
    pub entropy: *mut jpeg_entropy_decoder,
    pub cur_comp_info: [*mut jpeg_component_info; MAX_COMPS_IN_SCAN],
    pub comps_in_scan: c_int,
    pub Ss: c_int,
    pub Se: c_int,
    pub Ah: c_int,
    pub Al: c_int,
    pub blocks_in_MCU: c_int,
    pub MCU_membership: [c_int; 10],
    pub restart_interval: c_int,
    pub unread_marker: c_int,
    pub marker: *mut jpeg_marker_struct,
}

pub type j_decompress_ptr = *mut j_decompress_struct;
pub type j_common_ptr = *mut j_decompress_struct;
pub type JBLOCKROW = *mut [JCOEF; DCTSIZE2];

/* Derived data constructed for each Huffman table */
#[repr(C)]
pub struct d_derived_tbl {
    pub mincode: [INT32; 17],
    pub maxcode: [INT32; 18],
    pub valptr: [c_int; 17],
    pub pub_: *mut JHUFF_TBL,
    pub look_nbits: [c_int; 1 << HUFF_LOOKAHEAD],
    pub look_sym: [UINT8; 1 << HUFF_LOOKAHEAD],
}

/* Bitreading state saved across MCUs */
#[repr(C)]
pub struct bitread_perm_state {
    pub get_buffer: bit_buf_type,
    pub bits_left: c_int,
    pub printed_eod: boolean,
}

/* Bitreading working state within an MCU */
#[repr(C)]
pub struct bitread_working_state {
    pub next_input_byte: *const JOCTET,
    pub bytes_in_buffer: usize,
    pub unread_marker: c_int,
    pub get_buffer: bit_buf_type,
    pub bits_left: c_int,
    pub cinfo: j_decompress_ptr,
    pub printed_eod_ptr: *mut boolean,
}

/*
 * Expanded entropy decoder object for Huffman decoding.
 *
 * The savable_state subrecord contains fields that change within an MCU,
 * but must not be updated permanently until we complete the MCU.
 */
#[repr(C)]
struct savable_state {
    last_dc_val: [c_int; MAX_COMPS_IN_SCAN],
}

/* This macro is to work around compilers with missing or broken
 * structure assignment.  You'll need to fix this code if you have
 * such a compiler and you change MAX_COMPS_IN_SCAN.
 */
#[inline]
fn ASSIGN_STATE(dest: &mut savable_state, src: &savable_state) {
    dest.last_dc_val = src.last_dc_val;
}

#[repr(C)]
struct huff_entropy_decoder {
    pub_: jpeg_entropy_decoder,
    /* These fields are loaded into local variables at start of each MCU.
     * In case of suspension, we exit WITHOUT updating them.
     */
    bitstate: bitread_perm_state,
    saved: savable_state,

    /* These fields are NOT loaded into local working state. */
    restarts_to_go: c_int,

    /* Pointers to derived tables (these workspaces have image lifespan) */
    dc_derived_tbls: [*mut d_derived_tbl; NUM_HUFF_TBLS],
    ac_derived_tbls: [*mut d_derived_tbl; NUM_HUFF_TBLS],
}

type huff_entropy_ptr = *mut huff_entropy_decoder;

// ============================================================================
// Helper macros and functions
// ============================================================================

/// SIZEOF(T) - Get size of type T as usize
#[inline]
const fn SIZEOF<T>() -> usize {
    size_of::<T>()
}

/// MEMZERO(target, size) - Zero out memory region
#[inline]
unsafe fn MEMZERO(target: *mut c_void, size: usize) {
    core::ptr::write_bytes(target as *mut u8, 0, size);
}

/* Figure C.1: make table of Huffman code length for each symbol */
/* Note that this is in code-length order. */

/* Figure C.2: generate the codes themselves */
/* Note that this is in code-length order. */

/* Figure F.15: generate decoding tables for bit-sequential decoding */

/* Compute lookahead tables to speed up decoding. */
/* First we set all the table entries to 0, indicating "too long"; */
/* then we iterate through the Huffman codes that are short enough and */
/* fill in all the entries that correspond to bit sequences starting */
/* with that code. */

/*
 * Initialize for a Huffman-compressed scan.
 */

pub unsafe fn start_pass_huff_decoder(cinfo: j_decompress_ptr) {
    let entropy = (*cinfo).entropy as huff_entropy_ptr;
    let mut ci: c_int;
    let mut dctbl: c_int;
    let mut actbl: c_int;
    let mut compptr: *mut jpeg_component_info;

    /* Check that the scan parameters Ss, Se, Ah/Al are OK for sequential JPEG.
     * This ought to be an error condition, but we make it a warning because
     * there are some baseline files out there with all zeroes in these bytes.
     */
    if (*cinfo).Ss != 0 || (*cinfo).Se != (DCTSIZE2 as c_int) - 1 ||
        (*cinfo).Ah != 0 || (*cinfo).Al != 0
    {
        WARNMS(cinfo, JWRN_NOT_SEQUENTIAL);
    }

    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        compptr = (*cinfo).cur_comp_info[ci as usize];
        dctbl = (*compptr).dc_tbl_no;
        actbl = (*compptr).ac_tbl_no;
        /* Make sure requested tables are present */
        if dctbl < 0 || dctbl >= NUM_HUFF_TBLS as c_int ||
            (*cinfo).dc_huff_tbl_ptrs[dctbl as usize].is_null()
        {
            ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, dctbl);
        }
        if actbl < 0 || actbl >= NUM_HUFF_TBLS as c_int ||
            (*cinfo).ac_huff_tbl_ptrs[actbl as usize].is_null()
        {
            ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, actbl);
        }
        /* Compute derived values for Huffman tables */
        /* We may do this more than once for a table, but it's not expensive */
        jpeg_make_d_derived_tbl(
            cinfo,
            (*cinfo).dc_huff_tbl_ptrs[dctbl as usize],
            &mut (*entropy).dc_derived_tbls[dctbl as usize],
        );
        jpeg_make_d_derived_tbl(
            cinfo,
            (*cinfo).ac_huff_tbl_ptrs[actbl as usize],
            &mut (*entropy).ac_derived_tbls[actbl as usize],
        );
        /* Initialize DC predictions to 0 */
        (*entropy).saved.last_dc_val[ci as usize] = 0;

        ci += 1;
    }

    /* Initialize bitread state variables */
    (*entropy).bitstate.bits_left = 0;
    (*entropy).bitstate.get_buffer = 0; /* unnecessary, but keeps Purify quiet */
    (*entropy).bitstate.printed_eod = FALSE;

    /* Initialize restart counter */
    (*entropy).restarts_to_go = (*cinfo).restart_interval;
}

/*
 * Compute the derived values for a Huffman table.
 * Note this is also used by jdphuff.c.
 */

pub unsafe fn jpeg_make_d_derived_tbl(cinfo: j_decompress_ptr, htbl: *mut JHUFF_TBL, pdtbl: *mut *mut d_derived_tbl) {
    let mut dtbl: *mut d_derived_tbl;
    let mut p: c_int;
    let mut i: c_int;
    let mut l: c_int;
    let mut si: c_int;
    let mut lookbits: c_int;
    let mut ctr: c_int;
    let mut huffsize: [i8; 257] = [0; 257];
    let mut huffcode: [c_int; 257] = [0; 257];
    let mut code: c_int;

    /* Allocate a workspace if we haven't already done so. */
    if (*pdtbl).is_null() {
        *pdtbl = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            SIZEOF::<d_derived_tbl>(),
        ) as *mut d_derived_tbl;
    }
    dtbl = *pdtbl;
    (*dtbl).pub_ = htbl;

    /* Figure C.1: make table of Huffman code length for each symbol */
    /* Note that this is in code-length order. */

    p = 0;
    l = 1;
    while l <= 16 {
        i = 1;
        while i <= (*htbl).bits[l as usize] as c_int {
            huffsize[p as usize] = l as i8;
            p += 1;
            i += 1;
        }
        l += 1;
    }
    huffsize[p as usize] = 0;

    /* Figure C.2: generate the codes themselves */
    /* Note that this is in code-length order. */

    code = 0;
    si = huffsize[0] as c_int;
    p = 0;
    while huffsize[p as usize] != 0 {
        while (huffsize[p as usize] as c_int) == si {
            huffcode[p as usize] = code;
            code += 1;
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
            (*dtbl).mincode[l as usize] = huffcode[p as usize];
            p += (*htbl).bits[l as usize] as c_int;
            (*dtbl).maxcode[l as usize] = huffcode[(p - 1) as usize];
        } else {
            (*dtbl).maxcode[l as usize] = -1;
        }
        l += 1;
    }
    (*dtbl).maxcode[17] = 0xFFFFF;

    /* Compute lookahead tables to speed up decoding.
     * First we set all the table entries to 0, indicating "too long";
     * then we iterate through the Huffman codes that are short enough and
     * fill in all the entries that correspond to bit sequences starting
     * with that code.
     */

    MEMZERO(
        (*dtbl).look_nbits.as_mut_ptr() as *mut c_void,
        SIZEOF::<[c_int; 1 << HUFF_LOOKAHEAD]>(),
    );

    p = 0;
    l = 1;
    while l <= HUFF_LOOKAHEAD as c_int {
        i = 1;
        while i <= (*htbl).bits[l as usize] as c_int {
            lookbits = huffcode[p as usize] << (HUFF_LOOKAHEAD as c_int - l);
            ctr = 1 << (HUFF_LOOKAHEAD as c_int - l);
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

pub unsafe fn jpeg_fill_bit_buffer(
    state: *mut bitread_working_state,
    mut get_buffer: bit_buf_type,
    mut bits_left: c_int,
    nbits: c_int,
) -> boolean {
    /* Copy heavily used state fields into locals (hopefully registers) */
    let mut next_input_byte = (*state).next_input_byte;
    let mut bytes_in_buffer = (*state).bytes_in_buffer;
    let mut c: c_int;

    /* Attempt to load at least MIN_GET_BITS bits into get_buffer. */
    /* (It is assumed that no request will be for more than that many bits.) */

    while bits_left < MIN_GET_BITS as c_int {
        /* Attempt to read a byte */
        if (*state).unread_marker != 0 {
            if bits_left >= nbits {
                break;
            }
            /* There should be enough bits still left in the data segment; */
            /* if so, just break out of the outer while loop. */
            if *(*state).printed_eod_ptr == 0 {
                WARNMS((*state).cinfo, JWRN_HIT_MARKER);
                *(*state).printed_eod_ptr = TRUE;
            }
            c = 0;
        } else {
            if bytes_in_buffer == 0 {
                if !((*(*(*state).cinfo).src).fill_input_buffer)((*state).cinfo) != 0 {
                    (*state).next_input_byte = next_input_byte;
                    (*state).bytes_in_buffer = bytes_in_buffer;
                    (*state).get_buffer = get_buffer;
                    (*state).bits_left = bits_left;
                    return FALSE;
                }
                next_input_byte = (*(*(*state).cinfo).src).next_input_byte;
                bytes_in_buffer = (*(*(*state).cinfo).src).bytes_in_buffer;
            }
            bytes_in_buffer -= 1;
            c = *next_input_byte as c_int;
            next_input_byte = next_input_byte.offset(1);

            /* If it's 0xFF, check and discard stuffed zero byte */
            if c == 0xFF {
                loop {
                    if bytes_in_buffer == 0 {
                        if !((*(*(*state).cinfo).src).fill_input_buffer)((*state).cinfo) != 0 {
                            (*state).next_input_byte = next_input_byte;
                            (*state).bytes_in_buffer = bytes_in_buffer;
                            (*state).get_buffer = get_buffer;
                            (*state).bits_left = bits_left;
                            return FALSE;
                        }
                        next_input_byte = (*(*(*state).cinfo).src).next_input_byte;
                        bytes_in_buffer = (*(*(*state).cinfo).src).bytes_in_buffer;
                    }
                    bytes_in_buffer -= 1;
                    c = *next_input_byte as c_int;
                    next_input_byte = next_input_byte.offset(1);
                    if c != 0xFF {
                        break;
                    }
                }

                if c == 0 {
                    /* Found FF/00, which represents an FF data byte */
                    c = 0xFF;
                } else {
                    /* Oops, it's actually a marker indicating end of compressed data. */
                    /* Better put it back for use later */
                    (*state).unread_marker = c;

                    if bits_left >= nbits {
                        (*state).next_input_byte = next_input_byte;
                        (*state).bytes_in_buffer = bytes_in_buffer;
                        (*state).get_buffer = get_buffer;
                        (*state).bits_left = bits_left;
                        return TRUE;
                    }
                    /* Uh-oh.  Report corrupted data to user and stuff zeroes into
                     * the data stream, so that we can produce some kind of image.
                     * Note that this code will be repeated for each byte demanded
                     * for the rest of the segment.  We use a nonvolatile flag to ensure
                     * that only one warning message appears.
                     */
                    if *(*state).printed_eod_ptr == 0 {
                        WARNMS((*state).cinfo, JWRN_HIT_MARKER);
                        *(*state).printed_eod_ptr = TRUE;
                    }
                    c = 0;
                }
            }
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

    return TRUE;
}

/*
 * Out-of-line code for Huffman code decoding.
 * See jdhuff.h for info about usage.
 */

pub unsafe fn jpeg_huff_decode(
    state: *mut bitread_working_state,
    mut get_buffer: bit_buf_type,
    mut bits_left: c_int,
    htbl: *mut d_derived_tbl,
    min_bits: c_int,
) -> c_int {
    let mut l = min_bits;
    let mut code: INT32;

    /* HUFF_DECODE has determined that the code is at least min_bits */
    /* bits long, so fetch that many bits in one swoop. */

    if bits_left < l {
        if !jpeg_fill_bit_buffer(state, get_buffer, bits_left, l) != 0 {
            return -1;
        }
        get_buffer = (*state).get_buffer;
        bits_left = (*state).bits_left;
    }
    code = (get_buffer >> (bits_left - l)) & ((1 << l) - 1);
    bits_left -= l;

    /* Collect the rest of the Huffman code one bit at a time. */
    /* This is per Figure F.16 in the JPEG spec. */

    while code > (*htbl).maxcode[l as usize] {
        code <<= 1;
        if bits_left < 1 {
            if !jpeg_fill_bit_buffer(state, get_buffer, bits_left, 1) != 0 {
                return -1;
            }
            get_buffer = (*state).get_buffer;
            bits_left = (*state).bits_left;
        }
        code |= (get_buffer >> (bits_left - 1)) & 1;
        bits_left -= 1;
        l += 1;
    }

    /* Unload the local registers */
    (*state).get_buffer = get_buffer;
    (*state).bits_left = bits_left;

    /* With garbage input we may reach the sentinel value l = 17. */

    if l > 16 {
        WARNMS((*state).cinfo, JWRN_HUFF_BAD_CODE);
        return 0; /* fake a zero as the safest result */
    }

    return (*(*htbl).pub_).huffval[((*htbl).valptr[l as usize] + (code - (*htbl).mincode[l as usize])) as usize] as c_int;
}

/*
 * Figure F.12: extend sign bit.
 * On some machines, a shift and add will be faster than a table lookup.
 */

const EXTEND_TEST: [c_int; 16] = [
    /* entry n is 2**(n-1) */
    0, 0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080,
    0x0100, 0x0200, 0x0400, 0x0800, 0x1000, 0x2000, 0x4000,
];

const EXTEND_OFFSET: [c_int; 16] = [
    /* entry n is (-1 << n) + 1 */
    0, ((-1) << 1) + 1, ((-1) << 2) + 1, ((-1) << 3) + 1, ((-1) << 4) + 1,
    ((-1) << 5) + 1, ((-1) << 6) + 1, ((-1) << 7) + 1, ((-1) << 8) + 1,
    ((-1) << 9) + 1, ((-1) << 10) + 1, ((-1) << 11) + 1, ((-1) << 12) + 1,
    ((-1) << 13) + 1, ((-1) << 14) + 1, ((-1) << 15) + 1,
];

#[inline]
fn HUFF_EXTEND(x: c_int, s: c_int) -> c_int {
    if x < EXTEND_TEST[s as usize] {
        x + EXTEND_OFFSET[s as usize]
    } else {
        x
    }
}

/*
 * Check for a restart marker & resynchronize decoder.
 * Returns FALSE if must suspend.
 */

unsafe fn process_restart(cinfo: j_decompress_ptr) -> boolean {
    let entropy = (*cinfo).entropy as huff_entropy_ptr;
    let mut ci: c_int;

    /* Throw away any unused bits remaining in bit buffer; */
    /* include any full bytes in next_marker's count of discarded bytes */
    (*(*cinfo).marker).discarded_bytes += ((*entropy).bitstate.bits_left / 8) as usize;
    (*entropy).bitstate.bits_left = 0;

    /* Advance past the RSTn marker */
    if !((*(*cinfo).marker).read_restart_marker)(cinfo) != 0 {
        return FALSE;
    }

    /* Re-initialize DC predictions to 0 */
    ci = 0;
    while ci < (*cinfo).comps_in_scan {
        (*entropy).saved.last_dc_val[ci as usize] = 0;
        ci += 1;
    }

    /* Reset restart counter */
    (*entropy).restarts_to_go = (*cinfo).restart_interval;

    /* Next segment can get another out-of-data warning */
    (*entropy).bitstate.printed_eod = FALSE;

    return TRUE;
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

pub unsafe fn decode_mcu(cinfo: j_decompress_ptr, mcu_data: *mut JBLOCKROW) -> boolean {
    let entropy = (*cinfo).entropy as huff_entropy_ptr;
    let mut s: c_int;
    let mut k: c_int;
    let mut r: c_int;
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut block: JBLOCKROW;
    let mut get_buffer: bit_buf_type;
    let mut bits_left: c_int;
    let mut br_state: bitread_working_state;
    let mut state: savable_state;
    let mut dctbl: *mut d_derived_tbl;
    let mut actbl: *mut d_derived_tbl;
    let mut compptr: *mut jpeg_component_info;

    static JPEG_NATURAL_ORDER: [u8; 80] = [
        0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34,
        27, 20, 13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44,
        51, 58, 59, 52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63, 64, 64, 64, 64, 64, 64,
        64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    ];

    /* Process restart marker if needed; may have to suspend */
    if (*cinfo).restart_interval != 0 {
        if (*entropy).restarts_to_go == 0 {
            if !process_restart(cinfo) {
                return FALSE;
            }
        }
    }

    /* Load up working state */
    br_state.cinfo = cinfo;
    br_state.next_input_byte = (*(*cinfo).src).next_input_byte;
    br_state.bytes_in_buffer = (*(*cinfo).src).bytes_in_buffer;
    br_state.unread_marker = (*cinfo).unread_marker;
    get_buffer = (*entropy).bitstate.get_buffer;
    bits_left = (*entropy).bitstate.bits_left;
    br_state.printed_eod_ptr = &mut (*entropy).bitstate.printed_eod;

    state = savable_state {
        last_dc_val: (*entropy).saved.last_dc_val,
    };

    /* Outer loop handles each block in the MCU */

    blkn = 0;
    while blkn < (*cinfo).blocks_in_MCU {
        block = *mcu_data.offset(blkn as isize) as JBLOCKROW;
        ci = (*cinfo).MCU_membership[blkn as usize];
        compptr = (*cinfo).cur_comp_info[ci as usize];
        dctbl = (*entropy).dc_derived_tbls[(*compptr).dc_tbl_no as usize];
        actbl = (*entropy).ac_derived_tbls[(*compptr).ac_tbl_no as usize];

        /* Decode a single block's worth of coefficients */

        /* Section F.2.2.1: decode the DC coefficient difference */
        if bits_left < HUFF_LOOKAHEAD as c_int {
            if !jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) {
                (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                (*cinfo).unread_marker = br_state.unread_marker;
                (*entropy).bitstate.get_buffer = get_buffer;
                (*entropy).bitstate.bits_left = bits_left;
                return FALSE;
            }
            get_buffer = br_state.get_buffer;
            bits_left = br_state.bits_left;
            if bits_left < HUFF_LOOKAHEAD as c_int {
                s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, dctbl, 1);
                if s < 0 {
                    (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                    (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                    (*cinfo).unread_marker = br_state.unread_marker;
                    (*entropy).bitstate.get_buffer = get_buffer;
                    (*entropy).bitstate.bits_left = bits_left;
                    return FALSE;
                }
                get_buffer = br_state.get_buffer;
                bits_left = br_state.bits_left;
            } else {
                let look: c_int = (get_buffer >> (bits_left - HUFF_LOOKAHEAD as c_int)) & ((1 << HUFF_LOOKAHEAD as c_int) - 1);
                let nb: c_int = (*dctbl).look_nbits[look as usize];
                if nb != 0 {
                    bits_left -= nb;
                    s = (*dctbl).look_sym[look as usize] as c_int;
                } else {
                    s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, dctbl, HUFF_LOOKAHEAD as c_int + 1);
                    if s < 0 {
                        (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                        (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                        (*cinfo).unread_marker = br_state.unread_marker;
                        (*entropy).bitstate.get_buffer = get_buffer;
                        (*entropy).bitstate.bits_left = bits_left;
                        return FALSE;
                    }
                    get_buffer = br_state.get_buffer;
                    bits_left = br_state.bits_left;
                }
            }
        } else {
            let look: c_int = (get_buffer >> (bits_left - HUFF_LOOKAHEAD as c_int)) & ((1 << HUFF_LOOKAHEAD as c_int) - 1);
            let nb: c_int = (*dctbl).look_nbits[look as usize];
            if nb != 0 {
                bits_left -= nb;
                s = (*dctbl).look_sym[look as usize] as c_int;
            } else {
                s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, dctbl, HUFF_LOOKAHEAD as c_int + 1);
                if s < 0 {
                    (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                    (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                    (*cinfo).unread_marker = br_state.unread_marker;
                    (*entropy).bitstate.get_buffer = get_buffer;
                    (*entropy).bitstate.bits_left = bits_left;
                    return FALSE;
                }
                get_buffer = br_state.get_buffer;
                bits_left = br_state.bits_left;
            }
        }

        if s != 0 {
            if bits_left < s {
                if !jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) {
                    (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                    (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                    (*cinfo).unread_marker = br_state.unread_marker;
                    (*entropy).bitstate.get_buffer = get_buffer;
                    (*entropy).bitstate.bits_left = bits_left;
                    return FALSE;
                }
                get_buffer = br_state.get_buffer;
                bits_left = br_state.bits_left;
            }
            r = (get_buffer >> (bits_left - s)) & ((1 << s) - 1);
            bits_left -= s;
            s = HUFF_EXTEND(r, s);
        }

        /* Shortcut if component's values are not interesting */
        if (*compptr).component_needed == 0 {
            /* Section F.2.2.2: decode the AC coefficients */
            /* In this path we just discard the values */
            k = 1;
            while k < DCTSIZE2 as c_int {
                if bits_left < HUFF_LOOKAHEAD as c_int {
                    if !jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) {
                        (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                        (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                        (*cinfo).unread_marker = br_state.unread_marker;
                        (*entropy).bitstate.get_buffer = get_buffer;
                        (*entropy).bitstate.bits_left = bits_left;
                        return FALSE;
                    }
                    get_buffer = br_state.get_buffer;
                    bits_left = br_state.bits_left;
                    if bits_left < HUFF_LOOKAHEAD as c_int {
                        s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, 1);
                        if s < 0 {
                            (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                            (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                            (*cinfo).unread_marker = br_state.unread_marker;
                            (*entropy).bitstate.get_buffer = get_buffer;
                            (*entropy).bitstate.bits_left = bits_left;
                            return FALSE;
                        }
                        get_buffer = br_state.get_buffer;
                        bits_left = br_state.bits_left;
                    } else {
                        let look: c_int = (get_buffer >> (bits_left - HUFF_LOOKAHEAD as c_int)) & ((1 << HUFF_LOOKAHEAD as c_int) - 1);
                        let nb: c_int = (*actbl).look_nbits[look as usize];
                        if nb != 0 {
                            bits_left -= nb;
                            s = (*actbl).look_sym[look as usize] as c_int;
                        } else {
                            s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD as c_int + 1);
                            if s < 0 {
                                (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                                (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                                (*cinfo).unread_marker = br_state.unread_marker;
                                (*entropy).bitstate.get_buffer = get_buffer;
                                (*entropy).bitstate.bits_left = bits_left;
                                return FALSE;
                            }
                            get_buffer = br_state.get_buffer;
                            bits_left = br_state.bits_left;
                        }
                    }
                } else {
                    let look: c_int = (get_buffer >> (bits_left - HUFF_LOOKAHEAD as c_int)) & ((1 << HUFF_LOOKAHEAD as c_int) - 1);
                    let nb: c_int = (*actbl).look_nbits[look as usize];
                    if nb != 0 {
                        bits_left -= nb;
                        s = (*actbl).look_sym[look as usize] as c_int;
                    } else {
                        s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD as c_int + 1);
                        if s < 0 {
                            (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                            (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                            (*cinfo).unread_marker = br_state.unread_marker;
                            (*entropy).bitstate.get_buffer = get_buffer;
                            (*entropy).bitstate.bits_left = bits_left;
                            return FALSE;
                        }
                        get_buffer = br_state.get_buffer;
                        bits_left = br_state.bits_left;
                    }
                }

                r = s >> 4;
                s &= 15;

                if s != 0 {
                    k += r;
                    if bits_left < s {
                        if !jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) {
                            (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                            (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                            (*cinfo).unread_marker = br_state.unread_marker;
                            (*entropy).bitstate.get_buffer = get_buffer;
                            (*entropy).bitstate.bits_left = bits_left;
                            return FALSE;
                        }
                        get_buffer = br_state.get_buffer;
                        bits_left = br_state.bits_left;
                    }
                    r = (get_buffer >> (bits_left - s)) & ((1 << s) - 1);
                    bits_left -= s;
                    s = HUFF_EXTEND(r, s);
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
                while k < DCTSIZE2 as c_int {
                    if bits_left < HUFF_LOOKAHEAD as c_int {
                        if !jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) {
                            (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                            (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                            (*cinfo).unread_marker = br_state.unread_marker;
                            (*entropy).bitstate.get_buffer = get_buffer;
                            (*entropy).bitstate.bits_left = bits_left;
                            return FALSE;
                        }
                        get_buffer = br_state.get_buffer;
                        bits_left = br_state.bits_left;
                        if bits_left < HUFF_LOOKAHEAD as c_int {
                            s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, 1);
                            if s < 0 {
                                (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                                (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                                (*cinfo).unread_marker = br_state.unread_marker;
                                (*entropy).bitstate.get_buffer = get_buffer;
                                (*entropy).bitstate.bits_left = bits_left;
                                return FALSE;
                            }
                            get_buffer = br_state.get_buffer;
                            bits_left = br_state.bits_left;
                        } else {
                            let look: c_int = (get_buffer >> (bits_left - HUFF_LOOKAHEAD as c_int)) & ((1 << HUFF_LOOKAHEAD as c_int) - 1);
                            let nb: c_int = (*actbl).look_nbits[look as usize];
                            if nb != 0 {
                                bits_left -= nb;
                                s = (*actbl).look_sym[look as usize] as c_int;
                            } else {
                                s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD as c_int + 1);
                                if s < 0 {
                                    (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                                    (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                                    (*cinfo).unread_marker = br_state.unread_marker;
                                    (*entropy).bitstate.get_buffer = get_buffer;
                                    (*entropy).bitstate.bits_left = bits_left;
                                    return FALSE;
                                }
                                get_buffer = br_state.get_buffer;
                                bits_left = br_state.bits_left;
                            }
                        }
                    } else {
                        let look: c_int = (get_buffer >> (bits_left - HUFF_LOOKAHEAD as c_int)) & ((1 << HUFF_LOOKAHEAD as c_int) - 1);
                        let nb: c_int = (*actbl).look_nbits[look as usize];
                        if nb != 0 {
                            bits_left -= nb;
                            s = (*actbl).look_sym[look as usize] as c_int;
                        } else {
                            s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, actbl, HUFF_LOOKAHEAD as c_int + 1);
                            if s < 0 {
                                (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                                (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                                (*cinfo).unread_marker = br_state.unread_marker;
                                (*entropy).bitstate.get_buffer = get_buffer;
                                (*entropy).bitstate.bits_left = bits_left;
                                return FALSE;
                            }
                            get_buffer = br_state.get_buffer;
                            bits_left = br_state.bits_left;
                        }
                    }

                    r = s >> 4;
                    s &= 15;

                    if s != 0 {
                        k += r;
                        if bits_left < s {
                            if !jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) {
                                (*cinfo).src.as_mut().unwrap().next_input_byte = br_state.next_input_byte;
                                (*cinfo).src.as_mut().unwrap().bytes_in_buffer = br_state.bytes_in_buffer;
                                (*cinfo).unread_marker = br_state.unread_marker;
                                (*entropy).bitstate.get_buffer = get_buffer;
                                (*entropy).bitstate.bits_left = bits_left;
                                return FALSE;
                            }
                            get_buffer = br_state.get_buffer;
                            bits_left = br_state.bits_left;
                        }
                        r = (get_buffer >> (bits_left - s)) & ((1 << s) - 1);
                        bits_left -= s;
                        s = HUFF_EXTEND(r, s);
                        /* Output coefficient in natural (dezigzagged) order.
                         * Note: the extra entries in jpeg_natural_order[] will save us
                         * if k >= DCTSIZE2, which could happen if the data is corrupted.
                         */
                        (*block)[JPEG_NATURAL_ORDER[k as usize] as usize] = s as JCOEF;
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
    (*(*cinfo).src).next_input_byte = br_state.next_input_byte;
    (*(*cinfo).src).bytes_in_buffer = br_state.bytes_in_buffer;
    (*cinfo).unread_marker = br_state.unread_marker;
    (*entropy).bitstate.get_buffer = get_buffer;
    (*entropy).bitstate.bits_left = bits_left;

    ASSIGN_STATE(&mut (*entropy).saved, &state);

    /* Account for restart interval (no-op if not using restarts) */
    (*entropy).restarts_to_go -= 1;

    return TRUE;
}

/*
 * Module initialization routine for Huffman entropy decoding.
 */

pub unsafe fn jinit_huff_decoder(cinfo: j_decompress_ptr) {
    let mut entropy: huff_entropy_ptr;
    let mut i: c_int;

    entropy = ((*(*cinfo).mem).alloc_small)(
        cinfo as j_common_ptr,
        JPOOL_IMAGE,
        SIZEOF::<huff_entropy_decoder>(),
    ) as huff_entropy_ptr;

    (*cinfo).entropy = &mut (*entropy).pub_ as *mut jpeg_entropy_decoder;
    (*entropy).pub_.start_pass = Some(start_pass_huff_decoder);
    (*entropy).pub_.decode_mcu = Some(decode_mcu);

    /* Mark tables unallocated */
    i = 0;
    while i < NUM_HUFF_TBLS as c_int {
        (*entropy).dc_derived_tbls[i as usize] = core::ptr::null_mut();
        (*entropy).ac_derived_tbls[i as usize] = core::ptr::null_mut();
        i += 1;
    }
}

// ============================================================================
// Helper macros for error/warning handling (preserve original comments)
// ============================================================================

#[inline]
unsafe fn WARNMS(cinfo: j_decompress_ptr, code: c_int) {
    (*(*cinfo).err).msg_code = code;
    if let Some(emit_message) = (*(*cinfo).err).emit_message {
        emit_message(cinfo as *mut c_void, -1);
    }
}

#[inline]
unsafe fn ERREXIT1(cinfo: j_decompress_ptr, code: c_int, p1: c_int) {
    (*(*cinfo).err).msg_code = code;
    (*(*cinfo).err).msg_parm.i[0] = p1;
    if let Some(error_exit) = (*(*cinfo).err).error_exit {
        error_exit(cinfo as *mut c_void);
    }
}
