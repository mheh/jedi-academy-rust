/*
 * jcomapi.c
 *
 * Copyright (C) 1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains application interface routines that are used for both
 * compression and decompression.
 */

use core::ffi::{c_int, c_void};

// Local stubs for JPEG library types and constants.
// Full definitions would come from jpeglib.h and jinclude.h.
// These are declared to maintain structural coherence with the C code.

pub type boolean = c_int;
pub type UINT8 = u8;
pub type UINT16 = u16;

const DCTSIZE2: usize = 64;
const JPOOL_NUMPOOLS: c_int = 2;
const JPOOL_PERMANENT: c_int = 0;
const CSTATE_START: c_int = 100;
const DSTATE_START: c_int = 200;

/// JQUANT_TBL - Quantization table
#[repr(C)]
pub struct JQUANT_TBL {
	pub quantval: [UINT16; DCTSIZE2],
	/* quantization step for each coefficient */
	pub sent_table: boolean,
	/* TRUE when table has been output */
}

/// JHUFF_TBL - Huffman coding table
#[repr(C)]
pub struct JHUFF_TBL {
	pub bits: [UINT8; 17],
	/* bits[k] = # of symbols with codes of length k bits; bits[0] is unused */
	pub huffval: [UINT8; 256],
	/* The symbols, in order of incr code length */
	pub sent_table: boolean,
	/* TRUE when table has been output */
}

/// Opaque structures for JPEG structs
#[repr(C)]
pub struct jpeg_memory_mgr {
	pub alloc_small: Option<unsafe extern "C" fn(*mut c_void, c_int, usize) -> *mut c_void>,
	pub free_pool: Option<unsafe extern "C" fn(*mut c_void, c_int)>,
	pub self_destruct: Option<unsafe extern "C" fn(*mut c_void)>,
	// Other function pointers omitted for brevity in this stub
}

#[repr(C)]
pub struct jpeg_error_mgr {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_progress_mgr {
	_opaque: [u8; 0],
}

#[repr(C)]
pub struct jpeg_common_struct {
	pub err: *mut jpeg_error_mgr,
	pub mem: *mut jpeg_memory_mgr,
	pub progress: *mut jpeg_progress_mgr,
	pub is_decompressor: boolean,
	pub global_state: c_int,
}

pub type j_common_ptr = *mut jpeg_common_struct;

/*
 * Abort processing of a JPEG compression or decompression operation,
 * but don't destroy the object itself.
 *
 * For this, we merely clean up all the nonpermanent memory pools.
 * Note that temp files (virtual arrays) are not allowed to belong to
 * the permanent pool, so we will be able to close all temp files here.
 * Closing a data source or destination, if necessary, is the application's
 * responsibility.
 */

pub unsafe fn jpeg_abort(cinfo: j_common_ptr) {
	let mut pool: c_int;

	/* Releasing pools in reverse order might help avoid fragmentation
	 * with some (brain-damaged) malloc libraries.
	 */
	pool = JPOOL_NUMPOOLS - 1;
	while pool > JPOOL_PERMANENT {
		if let Some(free_pool_fn) = (*(*cinfo).mem).free_pool {
			free_pool_fn(cinfo as *mut c_void, pool);
		}
		pool -= 1;
	}

	/* Reset overall state for possible reuse of object */
	(*cinfo).global_state = if (*cinfo).is_decompressor != 0 { DSTATE_START } else { CSTATE_START };
}


/*
 * Destruction of a JPEG object.
 *
 * Everything gets deallocated except the master jpeg_compress_struct itself
 * and the error manager struct.  Both of these are supplied by the application
 * and must be freed, if necessary, by the application.  (Often they are on
 * the stack and so don't need to be freed anyway.)
 * Closing a data source or destination, if necessary, is the application's
 * responsibility.
 */

pub unsafe fn jpeg_destroy(cinfo: j_common_ptr) {
	/* We need only tell the memory manager to release everything. */
	/* NB: mem pointer is NULL if memory mgr failed to initialize. */
	if !(*cinfo).mem.is_null() {
		if let Some(self_destruct_fn) = (*(*cinfo).mem).self_destruct {
			self_destruct_fn(cinfo as *mut c_void);
		}
	}
	(*cinfo).mem = core::ptr::null_mut();	/* be safe if jpeg_destroy is called twice */
	(*cinfo).global_state = 0;	/* mark it destroyed */
}


/*
 * Convenience routines for allocating quantization and Huffman tables.
 * (Would jutils.c be a more reasonable place to put these?)
 */

pub unsafe fn jpeg_alloc_quant_table(cinfo: j_common_ptr) -> *mut JQUANT_TBL {
	let tbl: *mut JQUANT_TBL;

	if let Some(alloc_small_fn) = (*(*cinfo).mem).alloc_small {
		tbl = alloc_small_fn(
			cinfo as *mut c_void,
			JPOOL_PERMANENT,
			core::mem::size_of::<JQUANT_TBL>()
		) as *mut JQUANT_TBL;
		(*tbl).sent_table = 0;	/* make sure this is false in any new table */
		return tbl;
	}
	core::ptr::null_mut()
}


pub unsafe fn jpeg_alloc_huff_table(cinfo: j_common_ptr) -> *mut JHUFF_TBL {
	let tbl: *mut JHUFF_TBL;

	if let Some(alloc_small_fn) = (*(*cinfo).mem).alloc_small {
		tbl = alloc_small_fn(
			cinfo as *mut c_void,
			JPOOL_PERMANENT,
			core::mem::size_of::<JHUFF_TBL>()
		) as *mut JHUFF_TBL;
		(*tbl).sent_table = 0;	/* make sure this is false in any new table */
		return tbl;
	}
	core::ptr::null_mut()
}
