/*
 * jchuff.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains Huffman entropy encoding routines.
 *
 * Much of the complexity here has to do with supporting output suspension.
 * If the data destination module demands suspension, we want to be able to
 * back up to the start of the current MCU.  To do this, we copy state
 * variables into local working storage, and update them back to the
 * permanent JPEG objects only upon successful completion of an MCU.
 */

use core::ffi::{c_int, c_char};

// External types and functions from JPEG library
// These would be defined in the jpeg-6 headers (jinclude.h, jpeglib.h, jchuff.h)
pub type INT32 = i32;
pub type JOCTET = u8;
pub type JCOEFPTR = *mut i16;
pub type JBLOCKROW = *mut *mut i16;
pub type UINT8 = u8;
pub type boolean = c_int;

pub const TRUE: boolean = 1;
pub const FALSE: boolean = 0;

// Constants from jpeg-6
pub const MAX_COMPS_IN_SCAN: usize = 4;
pub const NUM_HUFF_TBLS: usize = 4;
pub const DCTSIZE2: usize = 64;
pub const MAX_CLEN: usize = 32;

// Macro replacements
pub const JPEG_RST0: u8 = 0xD0;

fn SIZEOF<T>(_: T) -> usize {
    core::mem::size_of::<T>()
}

fn SIZEOF_TYPE<T>() -> usize {
    core::mem::size_of::<T>()
}

// External types - stubs for JPEG library structures
#[repr(C)]
pub struct JHUFF_TBL {
    pub bits: [UINT8; 17],
    pub huffval: [UINT8; 256],
    pub sent_table: boolean,
}

#[repr(C)]
pub struct c_derived_tbl {
    pub ehufco: [u32; 256],
    pub ehufsi: [u8; 256],
}

#[repr(C)]
pub struct jpeg_component_info {
    pub dc_tbl_no: c_int,
    pub ac_tbl_no: c_int,
    // ... other fields omitted for brevity
}

#[repr(C)]
pub struct jpeg_destination_mgr {
    pub next_output_byte: *mut JOCTET,
    pub free_in_buffer: usize,
    pub empty_output_buffer: *const (),
}

#[repr(C)]
pub struct jpeg_entropy_encoder {
    pub start_pass: *const (),
    pub encode_mcu: *const (),
    pub finish_pass: *const (),
}

#[repr(C)]
pub struct j_compress_pub {
    pub entropy: *mut jpeg_entropy_encoder,
    pub dest: *mut jpeg_destination_mgr,
    pub restart_interval: u32,
    pub comps_in_scan: c_int,
    pub blocks_in_MCU: c_int,
    pub cur_comp_info: [*mut jpeg_component_info; MAX_COMPS_IN_SCAN],
    pub MCU_membership: [c_int; 10],
    pub dc_huff_tbl_ptrs: [*mut JHUFF_TBL; NUM_HUFF_TBLS],
    pub ac_huff_tbl_ptrs: [*mut JHUFF_TBL; NUM_HUFF_TBLS],
    pub mem: *mut (),
}

pub type j_compress_ptr = *mut j_compress_pub;
pub type j_common_ptr = *mut ();

// External functions
extern "C" {
    pub fn jpeg_make_c_derived_tbl(
        cinfo: j_compress_ptr,
        htbl: *mut JHUFF_TBL,
        pdtbl: *mut *mut c_derived_tbl,
    );
    pub fn jpeg_gen_optimal_table(
        cinfo: j_compress_ptr,
        htbl: *mut JHUFF_TBL,
        freq: *mut i64,
    );
    pub fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL;
    pub fn ERREXIT(cinfo: j_compress_ptr, msg_code: c_int);
    pub fn ERREXIT1(cinfo: j_compress_ptr, msg_code: c_int, p1: c_int);
}

static JPEG_NATURAL_ORDER: [u8; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10,
    17, 24, 32, 25, 18, 11, 4, 5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13, 6, 7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63,
];

/* Expanded entropy encoder object for Huffman encoding.
 *
 * The savable_state subrecord contains fields that change within an MCU,
 * but must not be updated permanently until we complete the MCU.
 */

#[repr(C)]
pub struct savable_state {
    pub put_buffer: INT32,                              /* current bit-accumulation buffer */
    pub put_bits: c_int,                                /* # of bits now in it */
    pub last_dc_val: [c_int; MAX_COMPS_IN_SCAN],       /* last DC coef for each component */
}

/* This macro is to work around compilers with missing or broken
 * structure assignment.  You'll need to fix this code if you have
 * such a compiler and you change MAX_COMPS_IN_SCAN.
 */

macro_rules! ASSIGN_STATE {
    ($dest:expr, $src:expr) => {
        $dest.put_buffer = $src.put_buffer;
        $dest.put_bits = $src.put_bits;
        $dest.last_dc_val[0] = $src.last_dc_val[0];
        $dest.last_dc_val[1] = $src.last_dc_val[1];
        $dest.last_dc_val[2] = $src.last_dc_val[2];
        $dest.last_dc_val[3] = $src.last_dc_val[3];
    };
}

#[repr(C)]
pub struct huff_entropy_encoder {
    pub pub_: jpeg_entropy_encoder,            /* public fields */
    pub saved: savable_state,                  /* Bit buffer & DC state at start of MCU */
    /* These fields are NOT loaded into local working state. */
    pub restarts_to_go: u32,                   /* MCUs left in this restart interval */
    pub next_restart_num: c_int,               /* next restart number to write (0-7) */
    /* Pointers to derived tables (these workspaces have image lifespan) */
    pub dc_derived_tbls: [*mut c_derived_tbl; NUM_HUFF_TBLS],
    pub ac_derived_tbls: [*mut c_derived_tbl; NUM_HUFF_TBLS],
    #[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
    pub dc_count_ptrs: [*mut i64; NUM_HUFF_TBLS],
    #[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
    pub ac_count_ptrs: [*mut i64; NUM_HUFF_TBLS],
}

pub type huff_entropy_ptr = *mut huff_entropy_encoder;

/* Working state while writing an MCU.
 * This struct contains all the fields that are needed by subroutines.
 */

#[repr(C)]
pub struct working_state {
    pub next_output_byte: *mut JOCTET,         /* => next byte to write in buffer */
    pub free_in_buffer: usize,                 /* # of byte spaces remaining in buffer */
    pub cur: savable_state,                    /* Current bit buffer & DC state */
    pub cinfo: j_compress_ptr,                 /* dump_buffer needs access to this */
}

/* Forward declarations */

/* Initialize for a Huffman-compressed scan.
 * If gather_statistics is TRUE, we do not output anything during the scan,
 * just count the Huffman symbols used and generate Huffman code tables.
 */

#[allow(non_snake_case)]
pub extern "C" fn start_pass_huff(cinfo: j_compress_ptr, gather_statistics: boolean) {
    let entropy = unsafe { cinfo as *mut huff_entropy_encoder };
    let entropy = unsafe { &mut *entropy };
    let mut ci: c_int;
    let mut dctbl: c_int;
    let mut actbl: c_int;
    let mut compptr: *mut jpeg_component_info;

    if gather_statistics != 0 {
        // ENTROPY_OPT_SUPPORTED case - not yet implemented
        unsafe { ERREXIT(cinfo, -1); } // JERR_NOT_COMPILED
    } else {
        // Set up normal encoding methods
        // entropy->pub.encode_mcu = encode_mcu_huff;
        // entropy->pub.finish_pass = finish_pass_huff;
    }

    ci = 0;
    while ci < unsafe { (*cinfo).comps_in_scan } {
        compptr = unsafe { (*cinfo).cur_comp_info[ci as usize] };
        dctbl = unsafe { (*compptr).dc_tbl_no };
        actbl = unsafe { (*compptr).ac_tbl_no };
        /* Make sure requested tables are present */
        /* (In gather mode, tables need not be allocated yet) */
        if dctbl < 0 || dctbl >= NUM_HUFF_TBLS as c_int
            || (unsafe { (*(*cinfo).dc_huff_tbl_ptrs[dctbl as usize]).is_null() }
                && gather_statistics == 0)
        {
            unsafe { ERREXIT1(cinfo, -1, dctbl); } // JERR_NO_HUFF_TABLE
        }
        if actbl < 0 || actbl >= NUM_HUFF_TBLS as c_int
            || (unsafe { (*(*cinfo).ac_huff_tbl_ptrs[actbl as usize]).is_null() }
                && gather_statistics == 0)
        {
            unsafe { ERREXIT1(cinfo, -1, actbl); } // JERR_NO_HUFF_TABLE
        }
        if gather_statistics != 0 {
            // ENTROPY_OPT_SUPPORTED - allocate and zero statistics tables
        } else {
            /* Compute derived values for Huffman tables */
            /* We may do this more than once for a table, but it's not expensive */
            unsafe {
                jpeg_make_c_derived_tbl(
                    cinfo,
                    (*cinfo).dc_huff_tbl_ptrs[dctbl as usize],
                    &mut entropy.dc_derived_tbls[dctbl as usize],
                );
                jpeg_make_c_derived_tbl(
                    cinfo,
                    (*cinfo).ac_huff_tbl_ptrs[actbl as usize],
                    &mut entropy.ac_derived_tbls[actbl as usize],
                );
            }
        }
        /* Initialize DC predictions to 0 */
        entropy.saved.last_dc_val[ci as usize] = 0;
        ci += 1;
    }

    /* Initialize bit buffer to empty */
    entropy.saved.put_buffer = 0;
    entropy.saved.put_bits = 0;

    /* Initialize restart stuff */
    entropy.restarts_to_go = unsafe { (*cinfo).restart_interval };
    entropy.next_restart_num = 0;
}

/*
 * Compute the derived values for a Huffman table.
 * Note this is also used by jcphuff.c.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_make_c_derived_tbl(
    cinfo: j_compress_ptr,
    htbl: *mut JHUFF_TBL,
    pdtbl: *mut *mut c_derived_tbl,
) {
    let mut dtbl: *mut c_derived_tbl;
    let mut p: c_int;
    let mut i: c_int;
    let mut l: c_int;
    let mut lastp: c_int;
    let mut si: c_int;
    let mut huffsize: [c_char; 257] = [0; 257];
    let mut huffcode: [u32; 257] = [0; 257];
    let mut code: u32;

    /* Allocate a workspace if we haven't already done so. */
    if unsafe { (*pdtbl).is_null() } {
        // Allocate memory - stub
        // *pdtbl = allocate c_derived_tbl
    }
    dtbl = unsafe { *pdtbl };

    /* Figure C.1: make table of Huffman code length for each symbol */
    /* Note that this is in code-length order. */

    p = 0;
    l = 1;
    while l <= 16 {
        i = 1;
        while i <= unsafe { (*htbl).bits[l as usize] } as c_int {
            huffsize[p as usize] = l as c_char;
            p += 1;
            i += 1;
        }
        l += 1;
    }
    huffsize[p as usize] = 0;
    lastp = p;

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

    /* Figure C.3: generate encoding tables */
    /* These are code and size indexed by symbol value */

    /* Set any codeless symbols to have code length 0;
     * this allows emit_bits to detect any attempt to emit such symbols.
     */
    unsafe {
        core::ptr::write_bytes(
            &mut (*dtbl).ehufsi as *mut [u8; 256] as *mut u8,
            0,
            256,
        );
    }

    p = 0;
    while p < lastp {
        let huffval_p = unsafe { (*htbl).huffval[p as usize] } as usize;
        unsafe {
            (*dtbl).ehufco[huffval_p] = huffcode[p as usize];
            (*dtbl).ehufsi[huffval_p] = huffsize[p as usize] as u8;
        }
        p += 1;
    }
}

/* Outputting bytes to the file */

/* Emit a byte, taking 'action' if must suspend. */
macro_rules! emit_byte {
    ($state:expr, $val:expr, $action:stmt) => {
        unsafe {
            *(*$state).next_output_byte = ($val) as JOCTET;
            (*$state).next_output_byte = (*$state).next_output_byte.add(1);
            (*$state).free_in_buffer = (*$state).free_in_buffer.wrapping_sub(1);
            if (*$state).free_in_buffer == 0 {
                if !dump_buffer($state) {
                    $action;
                }
            }
        }
    };
}

#[allow(non_snake_case)]
fn dump_buffer(state: *mut working_state) -> boolean {
    /* Empty the output buffer; return TRUE if successful, FALSE if must suspend */
    let dest = unsafe { (*(*state).cinfo).dest };

    if unsafe {
        let empty_fn = (*dest).empty_output_buffer;
        let empty_fn = core::mem::transmute::<
            *const (),
            extern "C" fn(j_compress_ptr) -> boolean,
        >(empty_fn);
        !empty_fn((*state).cinfo)
    } {
        return FALSE;
    }
    /* After a successful buffer dump, must reset buffer pointers */
    unsafe {
        (*state).next_output_byte = (*dest).next_output_byte;
        (*state).free_in_buffer = (*dest).free_in_buffer;
    }
    return TRUE;
}

/* Outputting bits to the file */

/* Only the right 24 bits of put_buffer are used; the valid bits are
 * left-justified in this part.  At most 16 bits can be passed to emit_bits
 * in one call, and we never retain more than 7 bits in put_buffer
 * between calls, so 24 bits are sufficient.
 */

#[inline]
#[allow(non_snake_case)]
fn emit_bits(state: *mut working_state, code: u32, mut size: c_int) -> boolean {
    /* Emit some bits; return TRUE if successful, FALSE if must suspend */
    /* This routine is heavily used, so it's worth coding tightly. */
    let mut put_buffer: INT32 = code as INT32;
    let mut put_bits: c_int = unsafe { (*state).cur.put_bits };

    /* if size is 0, caller used an invalid Huffman table entry */
    if size == 0 {
        unsafe { ERREXIT((*state).cinfo, -1); } // JERR_HUFF_MISSING_CODE
    }

    put_buffer &= (((1 as INT32) << size) - 1); /* mask off any extra bits in code */

    put_bits += size;        /* new number of bits in buffer */

    put_buffer = ((put_buffer as u32) << (24 - put_bits)) as INT32; /* align incoming bits */

    put_buffer |= unsafe { (*state).cur.put_buffer }; /* and merge with old buffer contents */

    while put_bits >= 8 {
        let c: c_int = (((put_buffer as u32) >> 16) & 0xFF) as c_int;

        emit_byte!(state, c, return FALSE);
        if c == 0xFF {
            /* need to stuff a zero byte? */
            emit_byte!(state, 0, return FALSE);
        }
        put_buffer = ((put_buffer as u32) << 8) as INT32;
        put_bits -= 8;
    }

    unsafe {
        (*state).cur.put_buffer = put_buffer; /* update state variables */
        (*state).cur.put_bits = put_bits;
    }

    return TRUE;
}

#[allow(non_snake_case)]
fn flush_bits(state: *mut working_state) -> boolean {
    if !emit_bits(state, 0x7F, 7) {
        /* fill any partial byte with ones */
        return FALSE;
    }
    unsafe {
        (*state).cur.put_buffer = 0;   /* and reset bit-buffer to empty */
        (*state).cur.put_bits = 0;
    }
    return TRUE;
}

/* Encode a single block's worth of coefficients */

#[allow(non_snake_case)]
fn encode_one_block(
    state: *mut working_state,
    block: JCOEFPTR,
    last_dc_val: c_int,
    dctbl: *mut c_derived_tbl,
    actbl: *mut c_derived_tbl,
) -> boolean {
    let mut temp: c_int;
    let mut temp2: c_int;
    let mut nbits: c_int;
    let mut k: c_int;
    let mut r: c_int;
    let mut i: c_int;

    /* Encode the DC coefficient difference per section F.1.2.1 */

    unsafe {
        temp = (*block.offset(0)) as c_int - last_dc_val;
    }
    temp2 = temp;

    if temp < 0 {
        temp = -temp;        /* temp is abs value of input */
        /* For a negative input, want temp2 = bitwise complement of abs(input) */
        /* This code assumes we are on a two's complement machine */
        temp2 -= 1;
    }

    /* Find the number of bits needed for the magnitude of the coefficient */
    nbits = 0;
    while temp != 0 {
        nbits += 1;
        temp >>= 1;
    }

    /* Emit the Huffman-coded symbol for the number of bits */
    if !emit_bits(
        state,
        unsafe { (*dctbl).ehufco[nbits as usize] },
        unsafe { (*dctbl).ehufsi[nbits as usize] } as c_int,
    ) {
        return FALSE;
    }

    /* Emit that number of bits of the value, if positive, */
    /* or the complement of its magnitude, if negative. */
    if nbits != 0 {
        /* emit_bits rejects calls with size 0 */
        if !emit_bits(state, temp2 as u32, nbits) {
            return FALSE;
        }
    }

    /* Encode the AC coefficients per section F.1.2.2 */

    r = 0;            /* r = run length of zeros */

    k = 1;
    while k < DCTSIZE2 as c_int {
        unsafe {
            temp = (*block.offset((JPEG_NATURAL_ORDER[k as usize] as isize))) as c_int;
        }
        if temp == 0 {
            r += 1;
        } else {
            /* if run length > 15, must emit special run-length-16 codes (0xF0) */
            while r > 15 {
                if !emit_bits(
                    state,
                    unsafe { (*actbl).ehufco[0xF0] },
                    unsafe { (*actbl).ehufsi[0xF0] } as c_int,
                ) {
                    return FALSE;
                }
                r -= 16;
            }

            temp2 = temp;
            if temp < 0 {
                temp = -temp;        /* temp is abs value of input */
                /* This code assumes we are on a two's complement machine */
                temp2 -= 1;
            }

            /* Find the number of bits needed for the magnitude of the coefficient */
            nbits = 1;        /* there must be at least one 1 bit */
            while {
                temp >>= 1;
                temp != 0
            } {
                nbits += 1;
            }

            /* Emit Huffman symbol for run length / number of bits */
            i = (r << 4) + nbits;
            if !emit_bits(
                state,
                unsafe { (*actbl).ehufco[i as usize] },
                unsafe { (*actbl).ehufsi[i as usize] } as c_int,
            ) {
                return FALSE;
            }

            /* Emit that number of bits of the value, if positive, */
            /* or the complement of its magnitude, if negative. */
            if !emit_bits(state, temp2 as u32, nbits) {
                return FALSE;
            }

            r = 0;
        }
        k += 1;
    }

    /* If the last coef(s) were zero, emit an end-of-block code */
    if r > 0 {
        if !emit_bits(
            state,
            unsafe { (*actbl).ehufco[0] },
            unsafe { (*actbl).ehufsi[0] } as c_int,
        ) {
            return FALSE;
        }
    }

    return TRUE;
}

/*
 * Emit a restart marker & resynchronize predictions.
 */

#[allow(non_snake_case)]
fn emit_restart(state: *mut working_state, restart_num: c_int) -> boolean {
    let mut ci: c_int;

    if !flush_bits(state) {
        return FALSE;
    }

    emit_byte!(state, 0xFF, return FALSE);
    emit_byte!(state, (JPEG_RST0 as c_int + restart_num) as JOCTET, return FALSE);

    /* Re-initialize DC predictions to 0 */
    ci = 0;
    while ci < unsafe { (*(*state).cinfo).comps_in_scan } {
        unsafe {
            (*state).cur.last_dc_val[ci as usize] = 0;
        }
        ci += 1;
    }

    /* The restart counter is not updated until we successfully write the MCU. */

    return TRUE;
}

/*
 * Encode and output one MCU's worth of Huffman-compressed coefficients.
 */

#[allow(non_snake_case)]
pub extern "C" fn encode_mcu_huff(cinfo: j_compress_ptr, MCU_data: JBLOCKROW) -> boolean {
    let entropy = unsafe { cinfo as *mut huff_entropy_encoder };
    let entropy = unsafe { &mut *entropy };
    let mut state: working_state = working_state {
        next_output_byte: core::ptr::null_mut(),
        free_in_buffer: 0,
        cur: savable_state {
            put_buffer: 0,
            put_bits: 0,
            last_dc_val: [0; MAX_COMPS_IN_SCAN],
        },
        cinfo: core::ptr::null_mut(),
    };
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    /* Load up working state */
    unsafe {
        state.next_output_byte = (*(*cinfo).dest).next_output_byte;
        state.free_in_buffer = (*(*cinfo).dest).free_in_buffer;
    }
    ASSIGN_STATE!(state.cur, entropy.saved);
    state.cinfo = cinfo;

    /* Emit restart marker if needed */
    if unsafe { (*cinfo).restart_interval } != 0 {
        if entropy.restarts_to_go == 0 {
            if !emit_restart(&mut state, entropy.next_restart_num) {
                return FALSE;
            }
        }
    }

    /* Encode the MCU data blocks */
    blkn = 0;
    while blkn < unsafe { (*cinfo).blocks_in_MCU } {
        unsafe {
            ci = (*cinfo).MCU_membership[blkn as usize];
            compptr = (*cinfo).cur_comp_info[ci as usize];
        }
        let dc_tbl_no = unsafe { (*compptr).dc_tbl_no };
        let ac_tbl_no = unsafe { (*compptr).ac_tbl_no };
        if !encode_one_block(
            &mut state,
            unsafe { *MCU_data.add(blkn as usize) },
            state.cur.last_dc_val[ci as usize],
            entropy.dc_derived_tbls[dc_tbl_no as usize],
            entropy.ac_derived_tbls[ac_tbl_no as usize],
        ) {
            return FALSE;
        }
        /* Update last_dc_val */
        unsafe {
            state.cur.last_dc_val[ci as usize] = *(*MCU_data.add(blkn as usize)).offset(0) as c_int;
        }
        blkn += 1;
    }

    /* Completed MCU, so update state */
    unsafe {
        (*(*cinfo).dest).next_output_byte = state.next_output_byte;
        (*(*cinfo).dest).free_in_buffer = state.free_in_buffer;
    }
    ASSIGN_STATE!(entropy.saved, state.cur);

    /* Update restart-interval state too */
    if unsafe { (*cinfo).restart_interval } != 0 {
        if entropy.restarts_to_go == 0 {
            entropy.restarts_to_go = unsafe { (*cinfo).restart_interval };
            entropy.next_restart_num += 1;
            entropy.next_restart_num &= 7;
        }
        entropy.restarts_to_go -= 1;
    }

    return TRUE;
}

/*
 * Finish up at the end of a Huffman-compressed scan.
 */

#[allow(non_snake_case)]
pub extern "C" fn finish_pass_huff(cinfo: j_compress_ptr) {
    let entropy = unsafe { cinfo as *mut huff_entropy_encoder };
    let entropy = unsafe { &mut *entropy };
    let mut state: working_state = working_state {
        next_output_byte: core::ptr::null_mut(),
        free_in_buffer: 0,
        cur: savable_state {
            put_buffer: 0,
            put_bits: 0,
            last_dc_val: [0; MAX_COMPS_IN_SCAN],
        },
        cinfo: core::ptr::null_mut(),
    };

    /* Load up working state ... flush_bits needs it */
    unsafe {
        state.next_output_byte = (*(*cinfo).dest).next_output_byte;
        state.free_in_buffer = (*(*cinfo).dest).free_in_buffer;
    }
    ASSIGN_STATE!(state.cur, entropy.saved);
    state.cinfo = cinfo;

    /* Flush out the last data */
    if !flush_bits(&mut state) {
        unsafe { ERREXIT(cinfo, -1); } // JERR_CANT_SUSPEND
    }

    /* Update state */
    unsafe {
        (*(*cinfo).dest).next_output_byte = state.next_output_byte;
        (*(*cinfo).dest).free_in_buffer = state.free_in_buffer;
    }
    ASSIGN_STATE!(entropy.saved, state.cur);
}

/*
 * Huffman coding optimization.
 *
 * This actually is optimization, in the sense that we find the best possible
 * Huffman table(s) for the given data.  We first scan the supplied data and
 * count the number of uses of each symbol that is to be Huffman-coded.
 * (This process must agree with the code above.)  Then we build an
 * optimal Huffman coding tree for the observed counts.
 *
 * The JPEG standard requires Huffman codes to be no more than 16 bits long.
 * If some symbols have a very small but nonzero probability, the Huffman tree
 * must be adjusted to meet the code length restriction.  We currently use
 * the adjustment method suggested in the JPEG spec.  This method is *not*
 * optimal; it may not choose the best possible limited-length code.  But
 * since the symbols involved are infrequently used, it's not clear that
 * going to extra trouble is worthwhile.
 */

#[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
#[allow(non_snake_case)]
fn htest_one_block(
    block: JCOEFPTR,
    last_dc_val: c_int,
    dc_counts: *mut i64,
    ac_counts: *mut i64,
) {
    let mut temp: c_int;
    let mut nbits: c_int;
    let mut k: c_int;
    let mut r: c_int;

    /* Encode the DC coefficient difference per section F.1.2.1 */

    unsafe {
        temp = (*block.offset(0)) as c_int - last_dc_val;
    }
    if temp < 0 {
        temp = -temp;
    }

    /* Find the number of bits needed for the magnitude of the coefficient */
    nbits = 0;
    while temp != 0 {
        nbits += 1;
        temp >>= 1;
    }

    /* Count the Huffman symbol for the number of bits */
    unsafe {
        *dc_counts.offset(nbits as isize) += 1;
    }

    /* Encode the AC coefficients per section F.1.2.2 */

    r = 0;            /* r = run length of zeros */

    k = 1;
    while k < DCTSIZE2 as c_int {
        unsafe {
            temp = (*block.offset((JPEG_NATURAL_ORDER[k as usize] as isize))) as c_int;
        }
        if temp == 0 {
            r += 1;
        } else {
            /* if run length > 15, must emit special run-length-16 codes (0xF0) */
            while r > 15 {
                unsafe {
                    *ac_counts.offset(0xF0) += 1;
                }
                r -= 16;
            }

            /* Find the number of bits needed for the magnitude of the coefficient */
            if temp < 0 {
                temp = -temp;
            }

            /* Find the number of bits needed for the magnitude of the coefficient */
            nbits = 1;        /* there must be at least one 1 bit */
            while {
                temp >>= 1;
                temp != 0
            } {
                nbits += 1;
            }

            /* Count Huffman symbol for run length / number of bits */
            unsafe {
                *ac_counts.offset(((r << 4) + nbits) as isize) += 1;
            }

            r = 0;
        }
        k += 1;
    }

    /* If the last coef(s) were zero, emit an end-of-block code */
    if r > 0 {
        unsafe {
            *ac_counts.offset(0) += 1;
        }
    }
}

/*
 * Trial-encode one MCU's worth of Huffman-compressed coefficients.
 * No data is actually output, so no suspension return is possible.
 */

#[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
#[allow(non_snake_case)]
pub extern "C" fn encode_mcu_gather(cinfo: j_compress_ptr, MCU_data: JBLOCKROW) -> boolean {
    let entropy = unsafe { cinfo as *mut huff_entropy_encoder };
    let entropy = unsafe { &mut *entropy };
    let mut blkn: c_int;
    let mut ci: c_int;
    let mut compptr: *mut jpeg_component_info;

    /* Take care of restart intervals if needed */
    if unsafe { (*cinfo).restart_interval } != 0 {
        if entropy.restarts_to_go == 0 {
            /* Re-initialize DC predictions to 0 */
            ci = 0;
            while ci < unsafe { (*cinfo).comps_in_scan } {
                entropy.saved.last_dc_val[ci as usize] = 0;
                ci += 1;
            }
            /* Update restart state */
            entropy.restarts_to_go = unsafe { (*cinfo).restart_interval };
        }
        entropy.restarts_to_go -= 1;
    }

    blkn = 0;
    while blkn < unsafe { (*cinfo).blocks_in_MCU } {
        unsafe {
            ci = (*cinfo).MCU_membership[blkn as usize];
            compptr = (*cinfo).cur_comp_info[ci as usize];
        }
        let dc_tbl_no = unsafe { (*compptr).dc_tbl_no };
        let ac_tbl_no = unsafe { (*compptr).ac_tbl_no };
        htest_one_block(
            unsafe { *MCU_data.add(blkn as usize) },
            entropy.saved.last_dc_val[ci as usize],
            entropy.dc_count_ptrs[dc_tbl_no as usize],
            entropy.ac_count_ptrs[ac_tbl_no as usize],
        );
        unsafe {
            entropy.saved.last_dc_val[ci as usize] =
                *(*MCU_data.add(blkn as usize)).offset(0) as c_int;
        }
        blkn += 1;
    }

    return TRUE;
}

/*
 * Generate the optimal coding for the given counts, fill htbl.
 * Note this is also used by jcphuff.c.
 */

#[allow(non_snake_case)]
pub extern "C" fn jpeg_gen_optimal_table(
    cinfo: j_compress_ptr,
    htbl: *mut JHUFF_TBL,
    freq: *mut i64,
) {
    let mut bits: [UINT8; MAX_CLEN + 1] = [0; MAX_CLEN + 1];    /* bits[k] = # of symbols with code length k */
    let mut codesize: [c_int; 257] = [0; 257];                   /* codesize[k] = code length of symbol k */
    let mut others: [c_int; 257] = [0; 257];                     /* next symbol in current branch of tree */
    let mut c1: c_int;
    let mut c2: c_int;
    let mut p: c_int;
    let mut i: c_int;
    let mut j: c_int;
    let mut v: i64;

    /* This algorithm is explained in section K.2 of the JPEG standard */

    for i in 0..=256 {
        others[i] = -1;       /* init links to empty */
    }

    unsafe {
        *freq.offset(256) = 1;
    }      /* make sure there is a nonzero count */
    /* Including the pseudo-symbol 256 in the Huffman procedure guarantees
     * that no real symbol is given code-value of all ones, because 256
     * will be placed in the largest codeword category.
     */

    /* Huffman's basic algorithm to assign optimal code lengths to symbols */

    loop {
        /* Find the smallest nonzero frequency, set c1 = its symbol */
        /* In case of ties, take the larger symbol number */
        c1 = -1;
        v = 1000000000i64;
        i = 0;
        while i <= 256 {
            if unsafe { *freq.offset(i as isize) } != 0
                && unsafe { *freq.offset(i as isize) } <= v
            {
                v = unsafe { *freq.offset(i as isize) };
                c1 = i;
            }
            i += 1;
        }

        /* Find the next smallest nonzero frequency, set c2 = its symbol */
        /* In case of ties, take the larger symbol number */
        c2 = -1;
        v = 1000000000i64;
        i = 0;
        while i <= 256 {
            if unsafe { *freq.offset(i as isize) } != 0
                && unsafe { *freq.offset(i as isize) } <= v
                && i != c1
            {
                v = unsafe { *freq.offset(i as isize) };
                c2 = i;
            }
            i += 1;
        }

        /* Done if we've merged everything into one frequency */
        if c2 < 0 {
            break;
        }

        /* Else merge the two counts/trees */
        unsafe {
            *freq.offset(c1 as isize) += *freq.offset(c2 as isize);
            *freq.offset(c2 as isize) = 0;
        }

        /* Increment the codesize of everything in c1's tree branch */
        codesize[c1 as usize] += 1;
        while others[c1 as usize] >= 0 {
            c1 = others[c1 as usize];
            codesize[c1 as usize] += 1;
        }

        others[c1 as usize] = c2;        /* chain c2 onto c1's tree branch */

        /* Increment the codesize of everything in c2's tree branch */
        codesize[c2 as usize] += 1;
        while others[c2 as usize] >= 0 {
            c2 = others[c2 as usize];
            codesize[c2 as usize] += 1;
        }
    }

    /* Now count the number of symbols of each code length */
    i = 0;
    while i <= 256 {
        if codesize[i as usize] != 0 {
            /* The JPEG standard seems to think that this can't happen, */
            /* but I'm paranoid... */
            if codesize[i as usize] > MAX_CLEN as c_int {
                unsafe { ERREXIT(cinfo, -1); } // JERR_HUFF_CLEN_OVERFLOW
            }

            bits[codesize[i as usize] as usize] += 1;
        }
        i += 1;
    }

    /* JPEG doesn't allow symbols with code lengths over 16 bits, so if the pure
     * Huffman procedure assigned any such lengths, we must adjust the coding.
     * Here is what the JPEG spec says about how this next bit works:
     * Since symbols are paired for the longest Huffman code, the symbols are
     * removed from this length category two at a time.  The prefix for the pair
     * (which is one bit shorter) is allocated to one of the pair; then,
     * skipping the BITS entry for that prefix length, a code word from the next
     * shortest nonzero BITS entry is converted into a prefix for two code words
     * one bit longer.
     */

    i = MAX_CLEN as c_int;
    while i > 16 {
        while bits[i as usize] > 0 {
            j = i - 2;        /* find length of new prefix to be used */
            while bits[j as usize] == 0 {
                j -= 1;
            }

            bits[i as usize] -= 2;        /* remove two symbols */
            bits[(i - 1) as usize] += 1;  /* one goes in this length */
            bits[(j + 1) as usize] += 2;  /* two new symbols in this length */
            bits[j as usize] -= 1;        /* symbol of this length is now a prefix */
        }
        i -= 1;
    }

    /* Remove the count for the pseudo-symbol 256 from the largest codelength */
    while bits[i as usize] == 0 {
        /* find largest codelength still in use */
        i -= 1;
    }
    bits[i as usize] -= 1;

    /* Return final symbol counts (only for lengths 0..16) */
    for j in 0..=16 {
        unsafe {
            (*htbl).bits[j] = bits[j];
        }
    }

    /* Return a list of the symbols sorted by code length */
    /* It's not real clear to me why we don't need to consider the codelength
     * changes made above, but the JPEG spec seems to think this works.
     */
    p = 0;
    i = 1;
    while i <= MAX_CLEN as c_int {
        j = 0;
        while j <= 255 {
            if codesize[j as usize] == i {
                unsafe {
                    (*htbl).huffval[p as usize] = j as UINT8;
                }
                p += 1;
            }
            j += 1;
        }
        i += 1;
    }

    /* Set sent_table FALSE so updated table will be written to JPEG file. */
    unsafe {
        (*htbl).sent_table = FALSE;
    }
}

/*
 * Finish up a statistics-gathering pass and create the new Huffman tables.
 */

#[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
#[allow(non_snake_case)]
pub extern "C" fn finish_pass_gather(cinfo: j_compress_ptr) {
    let entropy = unsafe { cinfo as *mut huff_entropy_encoder };
    let entropy = unsafe { &mut *entropy };
    let mut ci: c_int;
    let mut dctbl: c_int;
    let mut actbl: c_int;
    let mut compptr: *mut jpeg_component_info;
    let mut htblptr: *mut *mut JHUFF_TBL;
    let mut did_dc: [boolean; NUM_HUFF_TBLS] = [FALSE; NUM_HUFF_TBLS];
    let mut did_ac: [boolean; NUM_HUFF_TBLS] = [FALSE; NUM_HUFF_TBLS];

    /* It's important not to apply jpeg_gen_optimal_table more than once
     * per table, because it clobbers the input frequency counts!
     */

    ci = 0;
    while ci < unsafe { (*cinfo).comps_in_scan } {
        unsafe {
            compptr = (*cinfo).cur_comp_info[ci as usize];
            dctbl = (*compptr).dc_tbl_no;
            actbl = (*compptr).ac_tbl_no;
        }
        if did_dc[dctbl as usize] == FALSE {
            unsafe {
                htblptr = &mut (*cinfo).dc_huff_tbl_ptrs[dctbl as usize];
                if (*htblptr).is_null() {
                    *htblptr = jpeg_alloc_huff_table(cinfo as j_common_ptr);
                }
                jpeg_gen_optimal_table(
                    cinfo,
                    *htblptr,
                    entropy.dc_count_ptrs[dctbl as usize],
                );
            }
            did_dc[dctbl as usize] = TRUE;
        }
        if did_ac[actbl as usize] == FALSE {
            unsafe {
                htblptr = &mut (*cinfo).ac_huff_tbl_ptrs[actbl as usize];
                if (*htblptr).is_null() {
                    *htblptr = jpeg_alloc_huff_table(cinfo as j_common_ptr);
                }
                jpeg_gen_optimal_table(
                    cinfo,
                    *htblptr,
                    entropy.ac_count_ptrs[actbl as usize],
                );
            }
            did_ac[actbl as usize] = TRUE;
        }
        ci += 1;
    }
}

/*
 * Module initialization routine for Huffman entropy encoding.
 */

#[allow(non_snake_case)]
pub extern "C" fn jinit_huff_encoder(cinfo: j_compress_ptr) {
    let mut entropy: *mut huff_entropy_encoder;
    let mut i: c_int;

    /* Allocate a workspace for the entropy object */
    /* In C: (*cinfo->mem->alloc_small) would allocate memory */
    /* This is a stub - actual allocation would go through JPEG memory manager */
    unsafe {
        /* For now, assume entropy is allocated externally and passed in */
        entropy = (*cinfo).entropy as *mut huff_entropy_encoder;

        (*entropy).pub_.encode_mcu = core::ptr::null();
        (*entropy).pub_.finish_pass = core::ptr::null();
        (*entropy).pub_.start_pass = core::ptr::transmute::<
            extern "C" fn(j_compress_ptr, boolean),
            *const (),
        >(start_pass_huff);

        /* Mark tables unallocated */
        i = 0;
        while i < NUM_HUFF_TBLS as c_int {
            (*entropy).dc_derived_tbls[i as usize] = core::ptr::null_mut();
            (*entropy).ac_derived_tbls[i as usize] = core::ptr::null_mut();
            #[cfg(feature = "ENTROPY_OPT_SUPPORTED")]
            {
                (*entropy).dc_count_ptrs[i as usize] = core::ptr::null_mut();
                (*entropy).ac_count_ptrs[i as usize] = core::ptr::null_mut();
            }
            i += 1;
        }
    }
}
