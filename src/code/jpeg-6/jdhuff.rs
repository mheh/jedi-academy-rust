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

use core::ffi::{c_int, c_void};
use core::ptr;

// ============================================================================
// JPEG Library FFI Type Declarations
// ============================================================================

#[allow(non_camel_case_types)]
pub type j_decompress_ptr = *mut c_void;
#[allow(non_camel_case_types)]
pub type j_common_ptr = *mut c_void;
#[allow(non_camel_case_types)]
pub type JOCTET = u8;
#[allow(non_camel_case_types)]
pub type JBLOCKROW = *mut c_void;
#[allow(non_camel_case_types)]
pub type JCOEF = i16;
#[allow(non_camel_case_types)]
pub type boolean = c_int;
#[allow(non_camel_case_types)]
pub type JDIMENSION = u32;

const TRUE: c_int = 1;
const FALSE: c_int = 0;

const NUM_HUFF_TBLS: usize = 4;
const MAX_COMPS_IN_SCAN: usize = 4;
const DCTSIZE2: usize = 64;

// Constant from jdhuff.h
const HUFF_LOOKAHEAD: c_int = 8;
const BIT_BUF_SIZE: c_int = 32;

/* Selector for HUFF_EXTEND macro -- see jdhuff.h for usage */
const AVOID_TABLES: bool = false;

#[repr(C)]
pub struct bitread_perm_state {
    /* Bitreading state saved across MCUs */
    pub get_buffer: i32,     /* current bit-extraction buffer */
    pub bits_left: c_int,    /* # of unused bits in it */
    pub printed_eod: boolean, /* flag to suppress multiple warning msgs */
}

#[repr(C)]
pub struct bitread_working_state {
    /* Bitreading working state within an MCU */
    /* current data source state */
    pub next_input_byte: *const JOCTET, /* => next byte to read from source */
    pub bytes_in_buffer: usize,         /* # of bytes remaining in source buffer */
    pub unread_marker: c_int,           /* nonzero if we have hit a marker */
    /* bit input buffer --- note these values are kept in register variables,
     * not in this struct, inside the inner loops.
     */
    pub get_buffer: i32, /* current bit-extraction buffer */
    pub bits_left: c_int, /* # of unused bits in it */
    /* pointers needed by jpeg_fill_bit_buffer */
    pub cinfo: j_decompress_ptr, /* back link to decompress master record */
    pub printed_eod_ptr: *mut boolean, /* => flag in permanent state */
}

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

#[repr(C)]
pub struct JHUFF_TBL {
    /* Opaque Huffman table structure */
    pub bits: [u8; 17],   /* bits[k] = # of symbols with code length k */
    pub huffval: [u8; 256], /* The symbols, in order of incr code length */
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_component_info {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_entropy_decoder {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_marker_struct {
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_source_mgr {
    pub next_input_byte: *const JOCTET,
    pub bytes_in_buffer: usize,
    pub fill_input_buffer: Option<extern "C" fn(j_decompress_ptr) -> boolean>,
    pub skip_input_data: Option<extern "C" fn(j_decompress_ptr, c_int)>,
    pub resync_to_restart: Option<extern "C" fn(j_decompress_ptr, c_int) -> boolean>,
    pub term_source: Option<extern "C" fn(j_decompress_ptr)>,
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_decompress_struct {
    pub entropy: *mut jpeg_entropy_decoder,
    pub src: *mut jpeg_source_mgr,
    pub mem: *mut jpeg_memory_mgr,
    pub marker: *mut jpeg_marker_struct,
    pub comps_in_scan: c_int,
    pub Ss: c_int,
    pub Se: c_int,
    pub Ah: c_int,
    pub Al: c_int,
    pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; NUM_HUFF_TBLS],
    pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; NUM_HUFF_TBLS],
    pub cur_comp_info: [*mut jpeg_component_info; MAX_COMPS_IN_SCAN],
    pub restart_interval: c_int,
    pub unread_marker: c_int,
    pub blocks_in_MCU: c_int,
    pub MCU_membership: [c_int; 10],
    _priv: [u8; 0],
}

/*
 * Expanded entropy decoder object for Huffman decoding.
 *
 * The savable_state subrecord contains fields that change within an MCU,
 * but must not be updated permanently until we complete the MCU.
 */

#[repr(C)]
struct savable_state {
    last_dc_val: [c_int; MAX_COMPS_IN_SCAN], /* last DC coef for each component */
}

#[repr(C)]
struct huff_entropy_decoder {
    pub_: jpeg_entropy_decoder, /* public fields */

    /* These fields are loaded into local variables at start of each MCU.
     * In case of suspension, we exit WITHOUT updating them.
     */
    bitstate: bitread_perm_state, /* Bit buffer at start of MCU */
    saved: savable_state,         /* Other state at start of MCU */

    /* These fields are NOT loaded into local working state. */
    restarts_to_go: u32, /* MCUs left in this restart interval */

    /* Pointers to derived tables (these workspaces have image lifespan) */
    dc_derived_tbls: [*mut d_derived_tbl; NUM_HUFF_TBLS],
    ac_derived_tbls: [*mut d_derived_tbl; NUM_HUFF_TBLS],
}

type huff_entropy_ptr = *mut huff_entropy_decoder;

/* Error and warning message constants */
const JERR_NO_HUFF_TABLE: c_int = 37;
const JWRN_NOT_SEQUENTIAL: c_int = 173;
const JWRN_HIT_MARKER: c_int = 171;

const JPOOL_IMAGE: c_int = 0;
const MIN_GET_BITS: c_int = BIT_BUF_SIZE - 7;

extern "C" {
    fn WARNMS(cinfo: j_decompress_ptr, code: c_int);
    fn ERREXIT1(cinfo: j_decompress_ptr, code: c_int, p1: c_int);
    fn jpeg_natural_order(index: usize) -> usize;
}

/* Figure C.1 and C.2: make table of Huffman code length for each symbol */
/* Figure F.15: generate decoding tables for bit-sequential decoding */

/* These are the extend_test and extend_offset tables for HUFF_EXTEND macro */

#[allow(dead_code)]
static extend_test: [c_int; 16] = [
    /* entry n is 2**(n-1) */
    0, 0x0001, 0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0040, 0x0080,
    0x0100, 0x0200, 0x0400, 0x0800, 0x1000, 0x2000, 0x4000,
];

#[allow(dead_code)]
static extend_offset: [c_int; 16] = [
    /* entry n is (-1 << n) + 1 */
    0,
    ((-1) << 1) + 1,
    ((-1) << 2) + 1,
    ((-1) << 3) + 1,
    ((-1) << 4) + 1,
    ((-1) << 5) + 1,
    ((-1) << 6) + 1,
    ((-1) << 7) + 1,
    ((-1) << 8) + 1,
    ((-1) << 9) + 1,
    ((-1) << 10) + 1,
    ((-1) << 11) + 1,
    ((-1) << 12) + 1,
    ((-1) << 13) + 1,
    ((-1) << 14) + 1,
    ((-1) << 15) + 1,
];

/*
 * Initialize for a Huffman-compressed scan.
 */

extern "C" {
    fn start_pass_huff_decoder(cinfo: j_decompress_ptr);
}

#[no_mangle]
pub extern "C" fn start_pass_huff_decoder(cinfo: j_decompress_ptr) {
    unsafe {
        let entropy = cinfo as *mut huff_entropy_decoder;
        let mut ci: c_int;
        let mut dctbl: c_int;
        let mut actbl: c_int;
        let compptr: *mut jpeg_component_info;

        /* Check that the scan parameters Ss, Se, Ah/Al are OK for sequential JPEG.
         * This ought to be an error condition, but we make it a warning because
         * there are some baseline files out there with all zeroes in these bytes.
         */
        let cinfo_ref = &*(cinfo as *const jpeg_decompress_struct);
        if cinfo_ref.Ss != 0 || cinfo_ref.Se != (DCTSIZE2 - 1) as c_int
            || cinfo_ref.Ah != 0 || cinfo_ref.Al != 0
        {
            WARNMS(cinfo, JWRN_NOT_SEQUENTIAL);
        }

        ci = 0;
        while ci < cinfo_ref.comps_in_scan {
            compptr = cinfo_ref.cur_comp_info[ci as usize];
            let compptr_ref = &*(compptr);
            dctbl = (*compptr).dc_tbl_no;
            actbl = (*compptr).ac_tbl_no;
            /* Make sure requested tables are present */
            if dctbl < 0 || dctbl >= NUM_HUFF_TBLS as c_int
                || cinfo_ref.dc_huff_tbl_ptrs[dctbl as usize].is_null()
            {
                ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, dctbl);
            }
            if actbl < 0 || actbl >= NUM_HUFF_TBLS as c_int
                || cinfo_ref.ac_huff_tbl_ptrs[actbl as usize].is_null()
            {
                ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, actbl);
            }
            /* Compute derived values for Huffman tables */
            /* We may do this more than once for a table, but it's not expensive */
            jpeg_make_d_derived_tbl(
                cinfo,
                cinfo_ref.dc_huff_tbl_ptrs[dctbl as usize],
                &mut (*entropy).dc_derived_tbls[dctbl as usize],
            );
            jpeg_make_d_derived_tbl(
                cinfo,
                cinfo_ref.ac_huff_tbl_ptrs[actbl as usize],
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
        (*entropy).restarts_to_go = cinfo_ref.restart_interval as u32;
    }
}

/*
 * Compute the derived values for a Huffman table.
 * Note this is also used by jdphuff.c.
 */

#[no_mangle]
pub extern "C" fn jpeg_make_d_derived_tbl(
    cinfo: j_decompress_ptr,
    htbl: *mut JHUFF_TBL,
    pdtbl: &mut *mut d_derived_tbl,
) {
    unsafe {
        let mut dtbl: *mut d_derived_tbl;
        let mut p: c_int;
        let mut i: c_int;
        let mut l: c_int;
        let mut si: c_int;
        let mut lookbits: c_int;
        let mut ctr: c_int;
        let mut huffsize: [c_int; 257] = [0; 257];
        let mut huffcode: [u32; 257] = [0; 257];
        let mut code: u32;

        /* Allocate a workspace if we haven't already done so. */
        if pdtbl.is_null() {
            let cinfo_ref = &*(cinfo as *const jpeg_decompress_struct);
            let mem_mgr = &*(cinfo_ref.mem);
            if let Some(alloc_small) = mem_mgr.alloc_small {
                dtbl = alloc_small(
                    cinfo as j_common_ptr,
                    JPOOL_IMAGE,
                    core::mem::size_of::<d_derived_tbl>(),
                ) as *mut d_derived_tbl;
            } else {
                return;
            }
            *pdtbl = dtbl;
        } else {
            dtbl = *pdtbl;
        }

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
                (*dtbl).mincode[l as usize] = huffcode[p as usize] as i32; /* minimum code of length l */
                p += (*htbl).bits[l as usize] as c_int;
                (*dtbl).maxcode[l as usize] = huffcode[(p - 1) as usize] as i32; /* maximum code of length l */
            } else {
                (*dtbl).maxcode[l as usize] = -1; /* -1 if no codes of this length */
            }
            l += 1;
        }
        (*dtbl).maxcode[17] = 0xFFFFF; /* ensures jpeg_huff_decode terminates */

        /* Compute lookahead tables to speed up decoding.
         * First we set all the table entries to 0, indicating "too long";
         * then we iterate through the Huffman codes that are short enough and
         * fill in all the entries that correspond to bit sequences starting
         * with that code.
         */

        for idx in 0..256 {
            (*dtbl).look_nbits[idx] = 0;
        }

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

#[no_mangle]
pub extern "C" fn jpeg_fill_bit_buffer(
    state: *mut bitread_working_state,
    mut get_buffer: i32,
    mut bits_left: c_int,
    nbits: c_int,
) -> boolean {
    unsafe {
        /* Copy heavily used state fields into locals (hopefully registers) */
        let mut next_input_byte = (*state).next_input_byte;
        let mut bytes_in_buffer = (*state).bytes_in_buffer;
        let mut c: c_int;

        /* Attempt to load at least MIN_GET_BITS bits into get_buffer. */
        /* (It is assumed that no request will be for more than that many bits.) */

        while bits_left < MIN_GET_BITS {
            /* Attempt to read a byte */
            if (*state).unread_marker != 0 {
                goto_no_more_data! {
                    /* There should be enough bits still left in the data segment; */
                    /* if so, just break out of the outer while loop. */
                    if bits_left >= nbits {
                        break;
                    }
                    /* Uh-oh.  Report corrupted data to user and stuff zeroes into
                     * the data stream, so that we can produce some kind of image.
                     * Note that this code will be repeated for each byte demanded
                     * for the rest of the segment.  We use a nonvolatile flag to ensure
                     * that only one warning message appears.
                     */
                    if !(*(*state).printed_eod_ptr) != 0 {
                        WARNMS((*state).cinfo, JWRN_HIT_MARKER);
                        *(*state).printed_eod_ptr = TRUE;
                    }
                    c = 0; /* insert a zero byte into bit buffer */
                }
            }

            if bytes_in_buffer == 0 {
                let cinfo_ref = &*((*state).cinfo as *const jpeg_decompress_struct);
                if let Some(fill_input_buffer) = (*cinfo_ref.src).fill_input_buffer {
                    if fill_input_buffer((*state).cinfo) == FALSE {
                        return FALSE;
                    }
                } else {
                    return FALSE;
                }
                next_input_byte = (*cinfo_ref.src).next_input_byte;
                bytes_in_buffer = (*cinfo_ref.src).bytes_in_buffer;
            }
            bytes_in_buffer -= 1;
            c = *next_input_byte as c_int;
            next_input_byte = next_input_byte.offset(1);

            /* If it's 0xFF, check and discard stuffed zero byte */
            if c == 0xFF {
                loop {
                    if bytes_in_buffer == 0 {
                        let cinfo_ref = &*((*state).cinfo as *const jpeg_decompress_struct);
                        if let Some(fill_input_buffer) = (*cinfo_ref.src).fill_input_buffer {
                            if fill_input_buffer((*state).cinfo) == FALSE {
                                return FALSE;
                            }
                        } else {
                            return FALSE;
                        }
                        next_input_byte = (*cinfo_ref.src).next_input_byte;
                        bytes_in_buffer = (*cinfo_ref.src).bytes_in_buffer;
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

                    goto_no_more_data! {
                        /* There should be enough bits still left in the data segment; */
                        /* if so, just break out of the outer while loop. */
                        if bits_left >= nbits {
                            break;
                        }
                        /* Uh-oh.  Report corrupted data to user and stuff zeroes into
                         * the data stream, so that we can produce some kind of image.
                         * Note that this code will be repeated for each byte demanded
                         * for the rest of the segment.  We use a nonvolatile flag to ensure
                         * that only one warning message appears.
                         */
                        if !(*(*state).printed_eod_ptr) != 0 {
                            WARNMS((*state).cinfo, JWRN_HIT_MARKER);
                            *(*state).printed_eod_ptr = TRUE;
                        }
                        c = 0; /* insert a zero byte into bit buffer */
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
}

macro_rules! goto_no_more_data {
    ($block:block) => {
        $block
    };
}

/*
 * Out-of-line code for Huffman code decoding.
 * See jdhuff.h for info about usage.
 */

#[no_mangle]
pub extern "C" fn jpeg_huff_decode(
    state: *mut bitread_working_state,
    mut get_buffer: i32,
    mut bits_left: c_int,
    htbl: *mut d_derived_tbl,
    min_bits: c_int,
) -> c_int {
    unsafe {
        let mut l: c_int = min_bits;
        let mut code: i32;

        /* HUFF_DECODE has determined that the code is at least min_bits */
        /* bits long, so fetch that many bits in one swoop. */

        if bits_left < l {
            if jpeg_fill_bit_buffer(state, get_buffer, bits_left, l) == FALSE {
                return -1;
            }
            get_buffer = (*state).get_buffer;
            bits_left = (*state).bits_left;
        }
        bits_left -= l;
        code = ((get_buffer >> bits_left) & ((1 << l) - 1)) as i32;

        /* Collect the rest of the Huffman code one bit at a time. */
        /* This is per Figure F.16 in the JPEG spec. */

        while code > (*htbl).maxcode[l as usize] {
            code <<= 1;
            if bits_left < 1 {
                if jpeg_fill_bit_buffer(state, get_buffer, bits_left, 1) == FALSE {
                    return -1;
                }
                get_buffer = (*state).get_buffer;
                bits_left = (*state).bits_left;
            }
            bits_left -= 1;
            code |= ((get_buffer >> bits_left) & 1) as i32;
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

        return (*(*htbl).pub_).huffval
            [((*htbl).valptr[l as usize] + ((code - (*htbl).mincode[l as usize]) as c_int)) as usize]
            as c_int;
    }
}

const JWRN_HUFF_BAD_CODE: c_int = 170;

/*
 * Figure F.12: extend sign bit.
 * On some machines, a shift and add will be faster than a table lookup.
 */

#[inline]
fn HUFF_EXTEND(x: c_int, s: c_int) -> c_int {
    if !AVOID_TABLES {
        if x < extend_test[s as usize] {
            x + extend_offset[s as usize]
        } else {
            x
        }
    } else {
        if x < (1 << (s - 1)) {
            x + (((-1) << s) + 1)
        } else {
            x
        }
    }
}

/*
 * Check for a restart marker & resynchronize decoder.
 * Returns FALSE if must suspend.
 */

fn process_restart(cinfo: j_decompress_ptr) -> boolean {
    unsafe {
        let entropy = cinfo as *mut huff_entropy_decoder;
        let mut ci: c_int;

        let cinfo_ref = &*(cinfo as *const jpeg_decompress_struct);

        /* Throw away any unused bits remaining in bit buffer; */
        /* include any full bytes in next_marker's count of discarded bytes */
        (*cinfo_ref.marker).discarded_bytes += (*entropy).bitstate.bits_left / 8;
        (*entropy).bitstate.bits_left = 0;

        /* Advance past the RSTn marker */
        if let Some(read_restart_marker) = (*cinfo_ref.marker).read_restart_marker {
            if read_restart_marker(cinfo) == FALSE {
                return FALSE;
            }
        } else {
            return FALSE;
        }

        /* Re-initialize DC predictions to 0 */
        ci = 0;
        while ci < cinfo_ref.comps_in_scan {
            (*entropy).saved.last_dc_val[ci as usize] = 0;
            ci += 1;
        }

        /* Reset restart counter */
        (*entropy).restarts_to_go = cinfo_ref.restart_interval as u32;

        /* Next segment can get another out-of-data warning */
        (*entropy).bitstate.printed_eod = FALSE;

        return TRUE;
    }
}

#[repr(C)]
pub struct jpeg_marker_struct_full {
    pub discarded_bytes: c_int,
    pub read_restart_marker: Option<extern "C" fn(j_decompress_ptr) -> boolean>,
    _priv: [u8; 0],
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

#[no_mangle]
pub extern "C" fn decode_mcu(cinfo: j_decompress_ptr, MCU_data: *const *mut c_void) -> boolean {
    unsafe {
        let entropy = cinfo as *mut huff_entropy_decoder;
        let mut s: c_int;
        let mut k: c_int;
        let mut r: c_int;
        let mut blkn: c_int;
        let mut ci: c_int;
        let block: *mut JCOEF;
        let mut get_buffer: i32;
        let mut bits_left: c_int;
        let mut br_state: bitread_working_state;
        let mut state: savable_state;
        let dctbl: *mut d_derived_tbl;
        let actbl: *mut d_derived_tbl;
        let compptr: *mut jpeg_component_info;

        let cinfo_ref = &*(cinfo as *const jpeg_decompress_struct);

        /* Process restart marker if needed; may have to suspend */
        if cinfo_ref.restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                if process_restart(cinfo) == FALSE {
                    return FALSE;
                }
            }
        }

        /* Load up working state */
        /* BITREAD_LOAD_STATE */
        br_state.cinfo = cinfo;
        br_state.next_input_byte = (*cinfo_ref.src).next_input_byte;
        br_state.bytes_in_buffer = (*cinfo_ref.src).bytes_in_buffer;
        br_state.unread_marker = cinfo_ref.unread_marker;
        get_buffer = (*entropy).bitstate.get_buffer;
        bits_left = (*entropy).bitstate.bits_left;
        br_state.printed_eod_ptr = &mut (*entropy).bitstate.printed_eod;

        /* ASSIGN_STATE(state, entropy->saved) */
        state.last_dc_val = (*entropy).saved.last_dc_val;

        /* Outer loop handles each block in the MCU */

        blkn = 0;
        while blkn < cinfo_ref.blocks_in_MCU {
            block = *MCU_data.add(blkn as usize) as *mut JCOEF;
            ci = cinfo_ref.MCU_membership[blkn as usize];
            compptr = cinfo_ref.cur_comp_info[ci as usize];
            let dctbl_ptr = (*entropy).dc_derived_tbls[(*compptr).dc_tbl_no as usize];
            let actbl_ptr = (*entropy).ac_derived_tbls[(*compptr).ac_tbl_no as usize];

            /* Decode a single block's worth of coefficients */

            /* Section F.2.2.1: decode the DC coefficient difference */
            /* HUFF_DECODE macro logic - slow path for now */
            {
                let htbl = dctbl_ptr;
                let mut nb: c_int;
                let mut look: c_int;

                if bits_left < HUFF_LOOKAHEAD {
                    if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, 0) == FALSE {
                        return FALSE;
                    }
                    get_buffer = br_state.get_buffer;
                    bits_left = br_state.bits_left;
                    if bits_left < HUFF_LOOKAHEAD {
                        nb = 1;
                        s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, htbl, nb);
                        if s < 0 {
                            return FALSE;
                        }
                        get_buffer = br_state.get_buffer;
                        bits_left = br_state.bits_left;
                    } else {
                        look = (get_buffer >> (bits_left - HUFF_LOOKAHEAD))
                            & ((1 << HUFF_LOOKAHEAD) - 1);
                        if ((*htbl).look_nbits[look as usize]) != 0 {
                            nb = (*htbl).look_nbits[look as usize];
                            bits_left -= nb;
                            s = (*htbl).look_sym[look as usize] as c_int;
                        } else {
                            nb = HUFF_LOOKAHEAD + 1;
                            s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, htbl, nb);
                            if s < 0 {
                                return FALSE;
                            }
                            get_buffer = br_state.get_buffer;
                            bits_left = br_state.bits_left;
                        }
                    }
                } else {
                    look = (get_buffer >> (bits_left - HUFF_LOOKAHEAD))
                        & ((1 << HUFF_LOOKAHEAD) - 1);
                    if ((*htbl).look_nbits[look as usize]) != 0 {
                        nb = (*htbl).look_nbits[look as usize];
                        bits_left -= nb;
                        s = (*htbl).look_sym[look as usize] as c_int;
                    } else {
                        nb = HUFF_LOOKAHEAD + 1;
                        s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, htbl, nb);
                        if s < 0 {
                            return FALSE;
                        }
                        get_buffer = br_state.get_buffer;
                        bits_left = br_state.bits_left;
                    }
                }
            }

            if s != 0 {
                if bits_left < s {
                    if jpeg_fill_bit_buffer(&mut br_state, get_buffer, bits_left, s) == FALSE {
                        return FALSE;
                    }
                    get_buffer = br_state.get_buffer;
                    bits_left = br_state.bits_left;
                }
                bits_left -= s;
                r = ((get_buffer >> bits_left) & ((1 << s) - 1)) as c_int;
                s = HUFF_EXTEND(r, s);
            }

            /* Shortcut if component's values are not interesting */
            if (*compptr).component_needed == FALSE {
                /* skip to AC section */
            } else {
                /* Convert DC difference to actual value, update last_dc_val */
                s += state.last_dc_val[ci as usize];
                state.last_dc_val[ci as usize] = s;
                /* Output the DC coefficient (assumes jpeg_natural_order[0] = 0) */
                *block.add(0) = s as JCOEF;

                /* Do we need to decode the AC coefficients for this component? */
                if (*compptr).DCT_scaled_size > 1 {
                    /* Section F.2.2.2: decode the AC coefficients */
                    /* Since zeroes are skipped, output area must be cleared beforehand */
                    k = 1;
                    while k < DCTSIZE2 as c_int {
                        /* HUFF_DECODE macro for AC */
                        {
                            let htbl = actbl_ptr;
                            let mut nb: c_int;
                            let mut look: c_int;

                            if bits_left < HUFF_LOOKAHEAD {
                                if jpeg_fill_bit_buffer(
                                    &mut br_state,
                                    get_buffer,
                                    bits_left,
                                    0,
                                ) == FALSE
                                {
                                    return FALSE;
                                }
                                get_buffer = br_state.get_buffer;
                                bits_left = br_state.bits_left;
                                if bits_left < HUFF_LOOKAHEAD {
                                    nb = 1;
                                    s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, htbl, nb);
                                    if s < 0 {
                                        return FALSE;
                                    }
                                    get_buffer = br_state.get_buffer;
                                    bits_left = br_state.bits_left;
                                } else {
                                    look = (get_buffer >> (bits_left - HUFF_LOOKAHEAD))
                                        & ((1 << HUFF_LOOKAHEAD) - 1);
                                    if ((*htbl).look_nbits[look as usize]) != 0 {
                                        nb = (*htbl).look_nbits[look as usize];
                                        bits_left -= nb;
                                        s = (*htbl).look_sym[look as usize] as c_int;
                                    } else {
                                        nb = HUFF_LOOKAHEAD + 1;
                                        s = jpeg_huff_decode(
                                            &mut br_state,
                                            get_buffer,
                                            bits_left,
                                            htbl,
                                            nb,
                                        );
                                        if s < 0 {
                                            return FALSE;
                                        }
                                        get_buffer = br_state.get_buffer;
                                        bits_left = br_state.bits_left;
                                    }
                                }
                            } else {
                                look = (get_buffer >> (bits_left - HUFF_LOOKAHEAD))
                                    & ((1 << HUFF_LOOKAHEAD) - 1);
                                if ((*htbl).look_nbits[look as usize]) != 0 {
                                    nb = (*htbl).look_nbits[look as usize];
                                    bits_left -= nb;
                                    s = (*htbl).look_sym[look as usize] as c_int;
                                } else {
                                    nb = HUFF_LOOKAHEAD + 1;
                                    s = jpeg_huff_decode(
                                        &mut br_state,
                                        get_buffer,
                                        bits_left,
                                        htbl,
                                        nb,
                                    );
                                    if s < 0 {
                                        return FALSE;
                                    }
                                    get_buffer = br_state.get_buffer;
                                    bits_left = br_state.bits_left;
                                }
                            }
                        }

                        r = s >> 4;
                        s &= 15;

                        if s != 0 {
                            k += r;
                            if bits_left < s {
                                if jpeg_fill_bit_buffer(
                                    &mut br_state,
                                    get_buffer,
                                    bits_left,
                                    s,
                                ) == FALSE
                                {
                                    return FALSE;
                                }
                                get_buffer = br_state.get_buffer;
                                bits_left = br_state.bits_left;
                            }
                            bits_left -= s;
                            r = ((get_buffer >> bits_left) & ((1 << s) - 1)) as c_int;
                            s = HUFF_EXTEND(r, s);
                            /* Output coefficient in natural (dezigzagged) order.
                             * Note: the extra entries in jpeg_natural_order[] will save us
                             * if k >= DCTSIZE2, which could happen if the data is corrupted.
                             */
                            *block.add(jpeg_natural_order(k as usize) as usize) = s as JCOEF;
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
                    while k < DCTSIZE2 as c_int {
                        /* HUFF_DECODE macro for AC */
                        {
                            let htbl = actbl_ptr;
                            let mut nb: c_int;
                            let mut look: c_int;

                            if bits_left < HUFF_LOOKAHEAD {
                                if jpeg_fill_bit_buffer(
                                    &mut br_state,
                                    get_buffer,
                                    bits_left,
                                    0,
                                ) == FALSE
                                {
                                    return FALSE;
                                }
                                get_buffer = br_state.get_buffer;
                                bits_left = br_state.bits_left;
                                if bits_left < HUFF_LOOKAHEAD {
                                    nb = 1;
                                    s = jpeg_huff_decode(&mut br_state, get_buffer, bits_left, htbl, nb);
                                    if s < 0 {
                                        return FALSE;
                                    }
                                    get_buffer = br_state.get_buffer;
                                    bits_left = br_state.bits_left;
                                } else {
                                    look = (get_buffer >> (bits_left - HUFF_LOOKAHEAD))
                                        & ((1 << HUFF_LOOKAHEAD) - 1);
                                    if ((*htbl).look_nbits[look as usize]) != 0 {
                                        nb = (*htbl).look_nbits[look as usize];
                                        bits_left -= nb;
                                        s = (*htbl).look_sym[look as usize] as c_int;
                                    } else {
                                        nb = HUFF_LOOKAHEAD + 1;
                                        s = jpeg_huff_decode(
                                            &mut br_state,
                                            get_buffer,
                                            bits_left,
                                            htbl,
                                            nb,
                                        );
                                        if s < 0 {
                                            return FALSE;
                                        }
                                        get_buffer = br_state.get_buffer;
                                        bits_left = br_state.bits_left;
                                    }
                                }
                            } else {
                                look = (get_buffer >> (bits_left - HUFF_LOOKAHEAD))
                                    & ((1 << HUFF_LOOKAHEAD) - 1);
                                if ((*htbl).look_nbits[look as usize]) != 0 {
                                    nb = (*htbl).look_nbits[look as usize];
                                    bits_left -= nb;
                                    s = (*htbl).look_sym[look as usize] as c_int;
                                } else {
                                    nb = HUFF_LOOKAHEAD + 1;
                                    s = jpeg_huff_decode(
                                        &mut br_state,
                                        get_buffer,
                                        bits_left,
                                        htbl,
                                        nb,
                                    );
                                    if s < 0 {
                                        return FALSE;
                                    }
                                    get_buffer = br_state.get_buffer;
                                    bits_left = br_state.bits_left;
                                }
                            }
                        }

                        r = s >> 4;
                        s &= 15;

                        if s != 0 {
                            k += r;
                            if bits_left < s {
                                if jpeg_fill_bit_buffer(
                                    &mut br_state,
                                    get_buffer,
                                    bits_left,
                                    s,
                                ) == FALSE
                                {
                                    return FALSE;
                                }
                                get_buffer = br_state.get_buffer;
                                bits_left = br_state.bits_left;
                            }
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
        /* BITREAD_SAVE_STATE */
        (*cinfo_ref.src).next_input_byte = br_state.next_input_byte;
        (*cinfo_ref.src).bytes_in_buffer = br_state.bytes_in_buffer;
        /* Note: cannot update cinfo->unread_marker because it's const ref */
        (*entropy).bitstate.get_buffer = get_buffer;
        (*entropy).bitstate.bits_left = bits_left;

        /* ASSIGN_STATE(entropy->saved, state) */
        (*entropy).saved.last_dc_val = state.last_dc_val;

        /* Account for restart interval (no-op if not using restarts) */
        (*entropy).restarts_to_go = (*entropy).restarts_to_go.wrapping_sub(1);

        return TRUE;
    }
}

/*
 * Module initialization routine for Huffman entropy decoding.
 */

#[no_mangle]
pub extern "C" fn jinit_huff_decoder(cinfo: j_decompress_ptr) {
    unsafe {
        let entropy: *mut huff_entropy_decoder;
        let mut i: c_int;

        let cinfo_ref = &*(cinfo as *const jpeg_decompress_struct);
        if let Some(alloc_small) = (*cinfo_ref.mem).alloc_small {
            entropy = alloc_small(
                cinfo as j_common_ptr,
                JPOOL_IMAGE,
                core::mem::size_of::<huff_entropy_decoder>(),
            ) as *mut huff_entropy_decoder;
        } else {
            return;
        }

        let cinfo_mut = cinfo as *mut jpeg_decompress_struct;
        (*cinfo_mut).entropy = &mut (*entropy).pub_;

        let entropy_ref = &mut *entropy;
        entropy_ref.pub_.start_pass = Some(start_pass_huff_decoder);
        entropy_ref.pub_.decode_mcu = Some(decode_mcu);

        /* Mark tables unallocated */
        i = 0;
        while i < NUM_HUFF_TBLS as c_int {
            entropy_ref.dc_derived_tbls[i as usize] = ptr::null_mut();
            entropy_ref.ac_derived_tbls[i as usize] = ptr::null_mut();
            i += 1;
        }
    }
}

#[repr(C)]
pub struct jpeg_entropy_decoder_full {
    pub start_pass: Option<extern "C" fn(j_decompress_ptr)>,
    pub decode_mcu: Option<extern "C" fn(j_decompress_ptr, *const *mut c_void) -> boolean>,
    _priv: [u8; 0],
}

#[repr(C)]
pub struct jpeg_component_info_full {
    pub dc_tbl_no: c_int,
    pub ac_tbl_no: c_int,
    pub component_needed: boolean,
    pub DCT_scaled_size: c_int,
    _priv: [u8; 0],
}
