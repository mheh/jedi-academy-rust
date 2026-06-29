/*
 * jcphuff.c
 *
 * Copyright (C) 1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains Huffman entropy encoding routines for progressive JPEG.
 *
 * We do not support output suspension in this module, since the library
 * currently does not allow multiple-scan files to be written with output
 * suspension.
 */

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

// Stub types and externs for JPEG library
// These would be defined in other jpeg-6 modules in a complete port

extern "C" {
    // Type definitions from JPEG library
    // (These would be imported from other modules in full port)
    fn jpeg_make_c_derived_tbl(cinfo: *mut j_compress_ptr, htbl: *mut JHUFF_TBL, pdtbl: *mut *mut c_derived_tbl);
    fn jpeg_alloc_huff_table(ccommon: *mut j_common_ptr) -> *mut JHUFF_TBL;
    fn jpeg_gen_optimal_table(cinfo: *mut j_compress_ptr, htbl: *mut JHUFF_TBL, freq: *mut core::ffi::c_long);
}

// Forward declarations of stub types
#[repr(C)]
pub struct j_compress_struct {
    // Stub - full definition would be in jinclude.h
}

pub type j_compress_ptr = *mut j_compress_struct;

#[repr(C)]
pub struct j_common_struct {
    // Stub - full definition would be in jpeglib.h
}

pub type j_common_ptr = *mut j_common_struct;

#[repr(C)]
pub struct jpeg_destination_mgr {
    next_output_byte: *mut u8,
    free_in_buffer: usize,
    empty_output_buffer: extern "C" fn(*mut j_compress_ptr) -> c_int,
}

#[repr(C)]
pub struct jpeg_entropy_encoder {
    // Stub - this is the public part
}

#[repr(C)]
pub struct jpeg_component_info {
    // Stub - contains component-specific encoding parameters
}

#[repr(C)]
pub struct c_derived_tbl {
    // Stub - Huffman table structure
}

#[repr(C)]
pub struct JHUFF_TBL {
    // Stub - Huffman table
}

pub type JOCTET = u8;
pub type JBLOCKROW = *mut [i16; 64];
pub type boolean = c_int;

const TRUE: c_int = 1;
const FALSE: c_int = 0;

const MAX_COMPS_IN_SCAN: usize = 4;
const NUM_HUFF_TBLS: usize = 4;
const DCTSIZE2: usize = 64;

const JPEG_RST0: u8 = 0xD0;

const MAX_CORR_BITS: usize = 1000;

type INT32 = i32;

/* Expanded entropy encoder object for progressive Huffman encoding. */

#[repr(C)]
pub struct phuff_entropy_encoder {
    pub pub_: jpeg_entropy_encoder, /* public fields */

    /* Mode flag: TRUE for optimization, FALSE for actual data output */
    gather_statistics: boolean,

    /* Bit-level coding status.
     * next_output_byte/free_in_buffer are local copies of cinfo->dest fields.
     */
    next_output_byte: *mut JOCTET,	/* => next byte to write in buffer */
    free_in_buffer: usize,	/* # of byte spaces remaining in buffer */
    put_buffer: INT32,		/* current bit-accumulation buffer */
    put_bits: c_int,			/* # of bits now in it */
    cinfo: j_compress_ptr,		/* link to cinfo (needed for dump_buffer) */

    /* Coding status for DC components */
    last_dc_val: [c_int; MAX_COMPS_IN_SCAN], /* last DC coef for each component */

    /* Coding status for AC components */
    ac_tbl_no: c_int,		/* the table number of the single component */
    EOBRUN: u32,		/* run length of EOBs */
    BE: u32,		/* # of buffered correction bits before MCU */
    bit_buffer: *mut c_char,		/* buffer for correction bits (1 per char) */
    /* packing correction bits tightly would save some space but cost time... */

    restarts_to_go: u32,	/* MCUs left in this restart interval */
    next_restart_num: c_int,		/* next restart number to write (0-7) */

    /* Pointers to derived tables (these workspaces have image lifespan).
     * Since any one scan codes only DC or only AC, we only need one set
     * of tables, not one for DC and one for AC.
     */
    derived_tbls: [*mut c_derived_tbl; NUM_HUFF_TBLS],

    /* Statistics tables for optimization; again, one set is enough */
    count_ptrs: [*mut core::ffi::c_long; NUM_HUFF_TBLS],
}

pub type phuff_entropy_ptr = *mut phuff_entropy_encoder;

/* IRIGHT_SHIFT is like RIGHT_SHIFT, but works on int rather than INT32.
 * We assume that int right shift is unsigned if INT32 right shift is,
 * which should be safe.
 */

#[cfg(feature = "right_shift_is_unsigned")]
fn IRIGHT_SHIFT(x: c_int, shft: c_int) -> c_int {
    let ishift_temp = x;
    if ishift_temp < 0 {
        (ishift_temp >> shft) | (((!0) << (16 - shft)))
    } else {
        ishift_temp >> shft
    }
}

#[cfg(not(feature = "right_shift_is_unsigned"))]
fn IRIGHT_SHIFT(x: c_int, shft: c_int) -> c_int {
    x >> shft
}

/* Forward declarations */

fn encode_mcu_DC_first(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean;
fn encode_mcu_AC_first(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean;
fn encode_mcu_DC_refine(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean;
fn encode_mcu_AC_refine(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean;
fn finish_pass_phuff(cinfo: j_compress_ptr);
fn finish_pass_gather_phuff(cinfo: j_compress_ptr);


/*
 * Initialize for a Huffman-compressed scan using progressive JPEG.
 */

fn start_pass_phuff(cinfo: j_compress_ptr, gather_statistics: boolean) {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;
        let is_DC_band: boolean;
        let mut ci: c_int;
        let mut tbl: c_int;
        let mut compptr: *mut jpeg_component_info;

        (*entropy).cinfo = cinfo;
        (*entropy).gather_statistics = gather_statistics;

        is_DC_band = if (*cinfo).Ss == 0 { TRUE } else { FALSE };

        /* We assume jcmaster.c already validated the scan parameters. */

        /* Select execution routines */
        if (*cinfo).Ah == 0 {
            if is_DC_band != 0 {
                (*entropy).pub_.encode_mcu = Some(encode_mcu_DC_first);
            } else {
                (*entropy).pub_.encode_mcu = Some(encode_mcu_AC_first);
            }
        } else {
            if is_DC_band != 0 {
                (*entropy).pub_.encode_mcu = Some(encode_mcu_DC_refine);
            } else {
                (*entropy).pub_.encode_mcu = Some(encode_mcu_AC_refine);
                /* AC refinement needs a correction bit buffer */
                if (*entropy).bit_buffer.is_null() {
                    (*entropy).bit_buffer = ((*(*cinfo).mem).alloc_small)(
                        cinfo as j_common_ptr,
                        JPOOL_IMAGE,
                        MAX_CORR_BITS,
                    ) as *mut c_char;
                }
            }
        }
        if gather_statistics != 0 {
            (*entropy).pub_.finish_pass = Some(finish_pass_gather_phuff);
        } else {
            (*entropy).pub_.finish_pass = Some(finish_pass_phuff);
        }

        /* Only DC coefficients may be interleaved, so cinfo->comps_in_scan = 1
         * for AC coefficients.
         */
        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = (*cinfo).cur_comp_info[ci as usize];
            /* Initialize DC predictions to 0 */
            (*entropy).last_dc_val[ci as usize] = 0;
            /* Make sure requested tables are present */
            /* (In gather mode, tables need not be allocated yet) */
            if is_DC_band != 0 {
                if (*cinfo).Ah != 0 {
                    /* DC refinement needs no table */
                    ci += 1;
                    continue;
                }
                tbl = (*compptr).dc_tbl_no;
                if tbl < 0 || tbl >= NUM_HUFF_TBLS as c_int
                    || ((*(*cinfo).dc_huff_tbl_ptrs[tbl as usize]).is_null() && gather_statistics == 0) {
                    ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, tbl);
                }
            } else {
                (*entropy).ac_tbl_no = (*compptr).ac_tbl_no;
                tbl = (*compptr).ac_tbl_no;
                if tbl < 0 || tbl >= NUM_HUFF_TBLS as c_int
                    || ((*(*cinfo).ac_huff_tbl_ptrs[tbl as usize]).is_null() && gather_statistics == 0) {
                    ERREXIT1(cinfo, JERR_NO_HUFF_TABLE, tbl);
                }
            }
            if gather_statistics != 0 {
                /* Allocate and zero the statistics tables */
                /* Note that jpeg_gen_optimal_table expects 257 entries in each table! */
                if (*entropy).count_ptrs[tbl as usize].is_null() {
                    (*entropy).count_ptrs[tbl as usize] = ((*(*cinfo).mem).alloc_small)(
                        cinfo as j_common_ptr,
                        JPOOL_IMAGE,
                        257 * std::mem::size_of::<core::ffi::c_long>(),
                    ) as *mut core::ffi::c_long;
                }
                MEMZERO(
                    (*entropy).count_ptrs[tbl as usize] as *mut c_void,
                    257 * std::mem::size_of::<core::ffi::c_long>(),
                );
            } else {
                /* Compute derived values for Huffman tables */
                /* We may do this more than once for a table, but it's not expensive */
                if is_DC_band != 0 {
                    jpeg_make_c_derived_tbl(
                        cinfo,
                        (*cinfo).dc_huff_tbl_ptrs[tbl as usize],
                        &mut (*entropy).derived_tbls[tbl as usize],
                    );
                } else {
                    jpeg_make_c_derived_tbl(
                        cinfo,
                        (*cinfo).ac_huff_tbl_ptrs[tbl as usize],
                        &mut (*entropy).derived_tbls[tbl as usize],
                    );
                }
            }
            ci += 1;
        }

        /* Initialize AC stuff */
        (*entropy).EOBRUN = 0;
        (*entropy).BE = 0;

        /* Initialize bit buffer to empty */
        (*entropy).put_buffer = 0;
        (*entropy).put_bits = 0;

        /* Initialize restart stuff */
        (*entropy).restarts_to_go = (*cinfo).restart_interval;
        (*entropy).next_restart_num = 0;
    }
}


/* Outputting bytes to the file.
 * NB: these must be called only when actually outputting,
 * that is, entropy->gather_statistics == FALSE.
 */

/* Emit a byte */
#[inline]
fn emit_byte(entropy: phuff_entropy_ptr, val: JOCTET) {
    unsafe {
        let ptr = (*entropy).next_output_byte;
        *ptr = val;
        (*entropy).next_output_byte = ptr.offset(1);
        (*entropy).free_in_buffer -= 1;
        if (*entropy).free_in_buffer == 0 {
            dump_buffer(entropy);
        }
    }
}


fn dump_buffer(entropy: phuff_entropy_ptr) {
    /* Empty the output buffer; we do not support suspension in this module. */
    unsafe {
        let dest = (*(*entropy).cinfo).dest;

        if ((*dest).empty_output_buffer)((*entropy).cinfo) == 0 {
            ERREXIT((*entropy).cinfo, JERR_CANT_SUSPEND);
        }
        /* After a successful buffer dump, must reset buffer pointers */
        (*entropy).next_output_byte = (*dest).next_output_byte;
        (*entropy).free_in_buffer = (*dest).free_in_buffer;
    }
}


/* Outputting bits to the file */

/* Only the right 24 bits of put_buffer are used; the valid bits are
 * left-justified in this part.  At most 16 bits can be passed to emit_bits
 * in one call, and we never retain more than 7 bits in put_buffer
 * between calls, so 24 bits are sufficient.
 */

#[inline]
fn emit_bits(entropy: phuff_entropy_ptr, code: u32, size: c_int) {
    /* Emit some bits, unless we are in gather mode */
    unsafe {
        /* This routine is heavily used, so it's worth coding tightly. */
        let mut put_buffer = code as INT32;
        let mut put_bits = (*entropy).put_bits;

        /* if size is 0, caller used an invalid Huffman table entry */
        if size == 0 {
            ERREXIT((*entropy).cinfo, JERR_HUFF_MISSING_CODE);
        }

        if (*entropy).gather_statistics != 0 {
            return;			/* do nothing if we're only getting stats */
        }

        put_buffer &= (((1 as INT32) << size) - 1); /* mask off any extra bits in code */

        put_bits += size;		/* new number of bits in buffer */

        put_buffer <<= 24 - put_bits; /* align incoming bits */

        put_buffer |= (*entropy).put_buffer; /* and merge with old buffer contents */

        while put_bits >= 8 {
            let c = ((put_buffer >> 16) & 0xFF) as u8;

            emit_byte(entropy, c);
            if c == 0xFF {		/* need to stuff a zero byte? */
                emit_byte(entropy, 0);
            }
            put_buffer <<= 8;
            put_bits -= 8;
        }

        (*entropy).put_buffer = put_buffer; /* update variables */
        (*entropy).put_bits = put_bits;
    }
}


fn flush_bits(entropy: phuff_entropy_ptr) {
    unsafe {
        emit_bits(entropy, 0x7F, 7); /* fill any partial byte with ones */
        (*entropy).put_buffer = 0;     /* and reset bit-buffer to empty */
        (*entropy).put_bits = 0;
    }
}


/*
 * Emit (or just count) a Huffman symbol.
 */

#[inline]
fn emit_symbol(entropy: phuff_entropy_ptr, tbl_no: c_int, symbol: c_int) {
    unsafe {
        if (*entropy).gather_statistics != 0 {
            let ptr = (*entropy).count_ptrs[tbl_no as usize];
            *ptr.offset(symbol as isize) += 1;
        } else {
            let tbl = (*entropy).derived_tbls[tbl_no as usize];
            emit_bits(
                entropy,
                (*tbl).ehufco[symbol as usize] as u32,
                (*tbl).ehufsi[symbol as usize],
            );
        }
    }
}


/*
 * Emit bits from a correction bit buffer.
 */

fn emit_buffered_bits(entropy: phuff_entropy_ptr, bufstart: *mut c_char, nbits: u32) {
    unsafe {
        if (*entropy).gather_statistics != 0 {
            return;			/* no real work */
        }

        let mut buf = bufstart;
        let mut remaining = nbits;
        while remaining > 0 {
            emit_bits(entropy, *buf as u32, 1);
            buf = buf.offset(1);
            remaining -= 1;
        }
    }
}


/*
 * Emit any pending EOBRUN symbol.
 */

fn emit_eobrun(entropy: phuff_entropy_ptr) {
    unsafe {
        if (*entropy).EOBRUN > 0 {	/* if there is any pending EOBRUN */
            let mut temp = (*entropy).EOBRUN;
            let mut nbits = 0;
            while {
                temp >>= 1;
                temp != 0
            } {
                nbits += 1;
            }

            emit_symbol(entropy, (*entropy).ac_tbl_no, nbits << 4);
            if nbits != 0 {
                emit_bits(entropy, (*entropy).EOBRUN, nbits);
            }

            (*entropy).EOBRUN = 0;

            /* Emit any buffered correction bits */
            emit_buffered_bits(entropy, (*entropy).bit_buffer, (*entropy).BE);
            (*entropy).BE = 0;
        }
    }
}


/*
 * Emit a restart marker & resynchronize predictions.
 */

fn emit_restart(entropy: phuff_entropy_ptr, restart_num: c_int) {
    unsafe {
        emit_eobrun(entropy);

        if (*entropy).gather_statistics == 0 {
            flush_bits(entropy);
            emit_byte(entropy, 0xFF);
            emit_byte(entropy, (JPEG_RST0 as c_int + restart_num) as JOCTET);
        }

        if (*(*entropy).cinfo).Ss == 0 {
            /* Re-initialize DC predictions to 0 */
            let mut ci = 0;
            while ci < (*(*entropy).cinfo).comps_in_scan {
                (*entropy).last_dc_val[ci as usize] = 0;
                ci += 1;
            }
        } else {
            /* Re-initialize all AC-related fields to 0 */
            (*entropy).EOBRUN = 0;
            (*entropy).BE = 0;
        }
    }
}


/*
 * MCU encoding for DC initial scan (either spectral selection,
 * or first pass of successive approximation).
 */

fn encode_mcu_DC_first(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;
        let mut temp: c_int;
        let mut temp2: c_int;
        let mut nbits: c_int;
        let mut blkn: c_int;
        let mut ci: c_int;
        let Al = (*cinfo).Al;
        let mut block: JBLOCKROW;
        let mut compptr: *mut jpeg_component_info;

        (*entropy).next_output_byte = (*(*cinfo).dest).next_output_byte;
        (*entropy).free_in_buffer = (*(*cinfo).dest).free_in_buffer;

        /* Emit restart marker if needed */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                emit_restart(entropy, (*entropy).next_restart_num);
            }
        }

        /* Encode the MCU data blocks */
        blkn = 0;
        while blkn < (*cinfo).blocks_in_MCU {
            block = *MCU_data.offset(blkn as isize);
            ci = (*cinfo).MCU_membership[blkn as usize];
            compptr = (*cinfo).cur_comp_info[ci as usize];

            /* Compute the DC value after the required point transform by Al.
             * This is simply an arithmetic right shift.
             */
            temp2 = IRIGHT_SHIFT((*block)[0] as c_int, Al);

            /* DC differences are figured on the point-transformed values. */
            temp = temp2 - (*entropy).last_dc_val[ci as usize];
            (*entropy).last_dc_val[ci as usize] = temp2;

            /* Encode the DC coefficient difference per section G.1.2.1 */
            temp2 = temp;
            if temp < 0 {
                temp = -temp;		/* temp is abs value of input */
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

            /* Count/emit the Huffman-coded symbol for the number of bits */
            emit_symbol(entropy, (*compptr).dc_tbl_no, nbits);

            /* Emit that number of bits of the value, if positive, */
            /* or the complement of its magnitude, if negative. */
            if nbits != 0 {			/* emit_bits rejects calls with size 0 */
                emit_bits(entropy, temp2 as u32, nbits);
            }

            blkn += 1;
        }

        (*(*cinfo).dest).next_output_byte = (*entropy).next_output_byte;
        (*(*cinfo).dest).free_in_buffer = (*entropy).free_in_buffer;

        /* Update restart-interval state too */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                (*entropy).restarts_to_go = (*cinfo).restart_interval;
                (*entropy).next_restart_num += 1;
                (*entropy).next_restart_num &= 7;
            }
            (*entropy).restarts_to_go -= 1;
        }

        return TRUE;
    }
}


/*
 * MCU encoding for AC initial scan (either spectral selection,
 * or first pass of successive approximation).
 */

fn encode_mcu_AC_first(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;
        let mut temp: c_int;
        let mut temp2: c_int;
        let mut nbits: c_int;
        let mut r: c_int;
        let mut k: c_int;
        let Se = (*cinfo).Se;
        let Al = (*cinfo).Al;
        let block: JBLOCKROW;

        (*entropy).next_output_byte = (*(*cinfo).dest).next_output_byte;
        (*entropy).free_in_buffer = (*(*cinfo).dest).free_in_buffer;

        /* Emit restart marker if needed */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                emit_restart(entropy, (*entropy).next_restart_num);
            }
        }

        /* Encode the MCU data block */
        block = *MCU_data.offset(0);

        /* Encode the AC coefficients per section G.1.2.2, fig. G.3 */

        r = 0;			/* r = run length of zeros */

        k = (*cinfo).Ss;
        while k <= Se {
            if ({
                temp = (*block)[jpeg_natural_order[k as usize]];
                temp
            } == 0)
            {
                r += 1;
                k += 1;
                continue;
            }
            /* We must apply the point transform by Al.  For AC coefficients this
             * is an integer division with rounding towards 0.  To do this portably
             * in C, we shift after obtaining the absolute value; so the code is
             * interwoven with finding the abs value (temp) and output bits (temp2).
             */
            if temp < 0 {
                temp = -temp;		/* temp is abs value of input */
                temp >>= Al;		/* apply the point transform */
                /* For a negative coef, want temp2 = bitwise complement of abs(coef) */
                temp2 = !temp;
            } else {
                temp >>= Al;		/* apply the point transform */
                temp2 = temp;
            }
            /* Watch out for case that nonzero coef is zero after point transform */
            if temp == 0 {
                r += 1;
                k += 1;
                continue;
            }

            /* Emit any pending EOBRUN */
            if (*entropy).EOBRUN > 0 {
                emit_eobrun(entropy);
            }
            /* if run length > 15, must emit special run-length-16 codes (0xF0) */
            while r > 15 {
                emit_symbol(entropy, (*entropy).ac_tbl_no, 0xF0);
                r -= 16;
            }

            /* Find the number of bits needed for the magnitude of the coefficient */
            nbits = 1;			/* there must be at least one 1 bit */
            while {
                temp >>= 1;
                temp != 0
            } {
                nbits += 1;
            }

            /* Count/emit Huffman symbol for run length / number of bits */
            emit_symbol(entropy, (*entropy).ac_tbl_no, (r << 4) + nbits);

            /* Emit that number of bits of the value, if positive, */
            /* or the complement of its magnitude, if negative. */
            emit_bits(entropy, temp2 as u32, nbits);

            r = 0;			/* reset zero run length */

            k += 1;
        }

        if r > 0 {			/* If there are trailing zeroes, */
            (*entropy).EOBRUN += 1;		/* count an EOB */
            if (*entropy).EOBRUN == 0x7FFF {
                emit_eobrun(entropy);	/* force it out to avoid overflow */
            }
        }

        (*(*cinfo).dest).next_output_byte = (*entropy).next_output_byte;
        (*(*cinfo).dest).free_in_buffer = (*entropy).free_in_buffer;

        /* Update restart-interval state too */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                (*entropy).restarts_to_go = (*cinfo).restart_interval;
                (*entropy).next_restart_num += 1;
                (*entropy).next_restart_num &= 7;
            }
            (*entropy).restarts_to_go -= 1;
        }

        return TRUE;
    }
}


/*
 * MCU encoding for DC successive approximation refinement scan.
 * Note: we assume such scans can be multi-component, although the spec
 * is not very clear on the point.
 */

fn encode_mcu_DC_refine(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;
        let mut temp: c_int;
        let mut blkn: c_int;
        let Al = (*cinfo).Al;
        let block: JBLOCKROW;

        (*entropy).next_output_byte = (*(*cinfo).dest).next_output_byte;
        (*entropy).free_in_buffer = (*(*cinfo).dest).free_in_buffer;

        /* Emit restart marker if needed */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                emit_restart(entropy, (*entropy).next_restart_num);
            }
        }

        /* Encode the MCU data blocks */
        blkn = 0;
        while blkn < (*cinfo).blocks_in_MCU {
            let block = *MCU_data.offset(blkn as isize);

            /* We simply emit the Al'th bit of the DC coefficient value. */
            temp = (*block)[0] as c_int;
            emit_bits(entropy, ((temp >> Al) as u32) & 1, 1);

            blkn += 1;
        }

        (*(*cinfo).dest).next_output_byte = (*entropy).next_output_byte;
        (*(*cinfo).dest).free_in_buffer = (*entropy).free_in_buffer;

        /* Update restart-interval state too */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                (*entropy).restarts_to_go = (*cinfo).restart_interval;
                (*entropy).next_restart_num += 1;
                (*entropy).next_restart_num &= 7;
            }
            (*entropy).restarts_to_go -= 1;
        }

        return TRUE;
    }
}


/*
 * MCU encoding for AC successive approximation refinement scan.
 */

fn encode_mcu_AC_refine(cinfo: j_compress_ptr, MCU_data: *mut JBLOCKROW) -> boolean {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;
        let mut temp: c_int;
        let mut r: c_int;
        let mut k: c_int;
        let mut EOB: c_int;
        let mut BR_buffer: *mut c_char;
        let mut BR: u32;
        let Se = (*cinfo).Se;
        let Al = (*cinfo).Al;
        let block: JBLOCKROW;
        let mut absvalues: [c_int; DCTSIZE2] = [0; DCTSIZE2];

        (*entropy).next_output_byte = (*(*cinfo).dest).next_output_byte;
        (*entropy).free_in_buffer = (*(*cinfo).dest).free_in_buffer;

        /* Emit restart marker if needed */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                emit_restart(entropy, (*entropy).next_restart_num);
            }
        }

        /* Encode the MCU data block */
        let block = *MCU_data.offset(0);

        /* It is convenient to make a pre-pass to determine the transformed
         * coefficients' absolute values and the EOB position.
         */
        EOB = 0;
        k = (*cinfo).Ss;
        while k <= Se {
            temp = (*block)[jpeg_natural_order[k as usize]];
            /* We must apply the point transform by Al.  For AC coefficients this
             * is an integer division with rounding towards 0.  To do this portably
             * in C, we shift after obtaining the absolute value.
             */
            if temp < 0 {
                temp = -temp;		/* temp is abs value of input */
            }
            temp >>= Al;		/* apply the point transform */
            absvalues[k as usize] = temp;	/* save abs value for main pass */
            if temp == 1 {
                EOB = k;			/* EOB = index of last newly-nonzero coef */
            }
            k += 1;
        }

        /* Encode the AC coefficients per section G.1.2.3, fig. G.7 */

        r = 0;			/* r = run length of zeros */
        BR = 0;			/* BR = count of buffered bits added now */
        BR_buffer = (*entropy).bit_buffer.offset((*entropy).BE as isize); /* Append bits to buffer */

        k = (*cinfo).Ss;
        while k <= Se {
            temp = absvalues[k as usize];
            if temp == 0 {
                r += 1;
                k += 1;
                continue;
            }

            /* Emit any required ZRLs, but not if they can be folded into EOB */
            while r > 15 && k <= EOB {
                /* emit any pending EOBRUN and the BE correction bits */
                emit_eobrun(entropy);
                /* Emit ZRL */
                emit_symbol(entropy, (*entropy).ac_tbl_no, 0xF0);
                r -= 16;
                /* Emit buffered correction bits that must be associated with ZRL */
                emit_buffered_bits(entropy, BR_buffer, BR);
                BR_buffer = (*entropy).bit_buffer; /* BE bits are gone now */
                BR = 0;
            }

            /* If the coef was previously nonzero, it only needs a correction bit.
             * NOTE: a straight translation of the spec's figure G.7 would suggest
             * that we also need to test r > 15.  But if r > 15, we can only get here
             * if k > EOB, which implies that this coefficient is not 1.
             */
            if temp > 1 {
                /* The correction bit is the next bit of the absolute value. */
                *BR_buffer.offset(BR as isize) = ((temp & 1) as c_char);
                BR += 1;
                k += 1;
                continue;
            }

            /* Emit any pending EOBRUN and the BE correction bits */
            emit_eobrun(entropy);

            /* Count/emit Huffman symbol for run length / number of bits */
            emit_symbol(entropy, (*entropy).ac_tbl_no, (r << 4) + 1);

            /* Emit output bit for newly-nonzero coef */
            temp = if (*block)[jpeg_natural_order[k as usize]] < 0 { 0 } else { 1 };
            emit_bits(entropy, temp as u32, 1);

            /* Emit buffered correction bits that must be associated with this code */
            emit_buffered_bits(entropy, BR_buffer, BR);
            BR_buffer = (*entropy).bit_buffer; /* BE bits are gone now */
            BR = 0;
            r = 0;			/* reset zero run length */

            k += 1;
        }

        if r > 0 || BR > 0 {	/* If there are trailing zeroes, */
            (*entropy).EOBRUN += 1;		/* count an EOB */
            (*entropy).BE += BR;		/* concat my correction bits to older ones */
            /* We force out the EOB if we risk either:
             * 1. overflow of the EOB counter;
             * 2. overflow of the correction bit buffer during the next MCU.
             */
            if (*entropy).EOBRUN == 0x7FFF || (*entropy).BE > (MAX_CORR_BITS as u32 - DCTSIZE2 as u32 + 1) {
                emit_eobrun(entropy);
            }
        }

        (*(*cinfo).dest).next_output_byte = (*entropy).next_output_byte;
        (*(*cinfo).dest).free_in_buffer = (*entropy).free_in_buffer;

        /* Update restart-interval state too */
        if (*cinfo).restart_interval != 0 {
            if (*entropy).restarts_to_go == 0 {
                (*entropy).restarts_to_go = (*cinfo).restart_interval;
                (*entropy).next_restart_num += 1;
                (*entropy).next_restart_num &= 7;
            }
            (*entropy).restarts_to_go -= 1;
        }

        return TRUE;
    }
}


/*
 * Finish up at the end of a Huffman-compressed progressive scan.
 */

fn finish_pass_phuff(cinfo: j_compress_ptr) {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;

        (*entropy).next_output_byte = (*(*cinfo).dest).next_output_byte;
        (*entropy).free_in_buffer = (*(*cinfo).dest).free_in_buffer;

        /* Flush out any buffered data */
        emit_eobrun(entropy);
        flush_bits(entropy);

        (*(*cinfo).dest).next_output_byte = (*entropy).next_output_byte;
        (*(*cinfo).dest).free_in_buffer = (*entropy).free_in_buffer;
    }
}


/*
 * Finish up a statistics-gathering pass and create the new Huffman tables.
 */

fn finish_pass_gather_phuff(cinfo: j_compress_ptr) {
    unsafe {
        let entropy = cinfo as *mut phuff_entropy_encoder;
        let is_DC_band: boolean;
        let mut ci: c_int;
        let mut tbl: c_int;
        let mut compptr: *mut jpeg_component_info;
        let mut htblptr: *mut *mut JHUFF_TBL;
        let mut did: [boolean; NUM_HUFF_TBLS] = [0; NUM_HUFF_TBLS];

        /* Flush out buffered data (all we care about is counting the EOB symbol) */
        emit_eobrun(entropy);

        is_DC_band = if (*cinfo).Ss == 0 { TRUE } else { FALSE };

        /* It's important not to apply jpeg_gen_optimal_table more than once
         * per table, because it clobbers the input frequency counts!
         */
        MEMZERO(
            did.as_mut_ptr() as *mut c_void,
            std::mem::size_of_val(&did),
        );

        ci = 0;
        while ci < (*cinfo).comps_in_scan {
            compptr = (*cinfo).cur_comp_info[ci as usize];
            if is_DC_band != 0 {
                if (*cinfo).Ah != 0 {	/* DC refinement needs no table */
                    ci += 1;
                    continue;
                }
                tbl = (*compptr).dc_tbl_no;
            } else {
                tbl = (*compptr).ac_tbl_no;
            }
            if did[tbl as usize] == 0 {
                if is_DC_band != 0 {
                    htblptr = &mut (*cinfo).dc_huff_tbl_ptrs[tbl as usize];
                } else {
                    htblptr = &mut (*cinfo).ac_huff_tbl_ptrs[tbl as usize];
                }
                if (*htblptr).is_null() {
                    *htblptr = jpeg_alloc_huff_table(cinfo as j_common_ptr);
                }
                jpeg_gen_optimal_table(
                    cinfo,
                    *htblptr,
                    (*entropy).count_ptrs[tbl as usize],
                );
                did[tbl as usize] = TRUE;
            }
            ci += 1;
        }
    }
}


/*
 * Module initialization routine for progressive Huffman entropy encoding.
 */

pub fn jinit_phuff_encoder(cinfo: j_compress_ptr) {
    unsafe {
        let mut entropy: phuff_entropy_ptr;
        let mut i: c_int;

        entropy = ((*(*cinfo).mem).alloc_small)(
            cinfo as j_common_ptr,
            JPOOL_IMAGE,
            std::mem::size_of::<phuff_entropy_encoder>(),
        ) as phuff_entropy_ptr;
        (*cinfo).entropy = entropy as *mut jpeg_entropy_encoder;
        (*entropy).pub_.start_pass = Some(start_pass_phuff);

        /* Mark tables unallocated */
        i = 0;
        while i < NUM_HUFF_TBLS as c_int {
            (*entropy).derived_tbls[i as usize] = std::ptr::null_mut();
            (*entropy).count_ptrs[i as usize] = std::ptr::null_mut();
            i += 1;
        }
        (*entropy).bit_buffer = std::ptr::null_mut();	/* needed only in AC refinement scan */
    }
}


// Stub externs and constants for JPEG macros

// Stub error codes
const JERR_NO_HUFF_TABLE: c_int = 1;
const JERR_HUFF_MISSING_CODE: c_int = 2;
const JERR_CANT_SUSPEND: c_int = 3;

const JPOOL_IMAGE: c_int = 1;

fn ERREXIT(cinfo: j_compress_ptr, code: c_int) {
    // Stub - would call actual error handler
}

fn ERREXIT1(cinfo: j_compress_ptr, code: c_int, arg: c_int) {
    // Stub - would call actual error handler with argument
}

fn MEMZERO(ptr: *mut c_void, size: usize) {
    unsafe {
        core::ptr::write_bytes(ptr as *mut u8, 0, size);
    }
}

// Stub extern for jpeg_natural_order
extern "C" {
    #[link_name = "jpeg_natural_order"]
    pub static jpeg_natural_order: [c_int; DCTSIZE2];
}
