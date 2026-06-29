/*
 * jmemsys.h
 *
 * Copyright (C) 1992-1994, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This include file defines the interface between the system-independent
 * and system-dependent portions of the JPEG memory manager.  No other
 * modules need include it.  (The system-independent portion is jmemmgr.c;
 * there are several different versions of the system-dependent portion.)
 *
 * This file works as-is for the system-dependent memory managers supplied
 * in the IJG distribution.  You may need to modify it if you write a
 * custom memory manager.  If system-dependent changes are needed in
 * this file, the best method is to #ifdef them based on a configuration
 * symbol supplied in jconfig.h, as we have done with USE_MSDOS_MEMMGR.
 */

use core::ffi::{c_char, c_void};

/* Stub opaque type for JPEG common structure */
pub struct jpeg_common_struct;
pub type j_common_ptr = *mut jpeg_common_struct;

/* Short forms of external names for systems with brain-damaged linkers. */

#[cfg(feature = "NEED_SHORT_EXTERNAL_NAMES")]
pub mod short_external_names {
    // #define jpeg_get_small		jGetSmall
    // #define jpeg_free_small		jFreeSmall
    // #define jpeg_get_large		jGetLarge
    // #define jpeg_free_large		jFreeLarge
    // #define jpeg_mem_available	jMemAvail
    // #define jpeg_open_backing_store	jOpenBackStore
    // #define jpeg_mem_init		jMemInit
    // #define jpeg_mem_term		jMemTerm
}

/*
 * These two functions are used to allocate and release small chunks of
 * memory.  (Typically the total amount requested through jpeg_get_small is
 * no more than 20K or so; this will be requested in chunks of a few K each.)
 * Behavior should be the same as for the standard library functions malloc
 * and free; in particular, jpeg_get_small must return NULL on failure.
 * On most systems, these ARE malloc and free.  jpeg_free_small is passed the
 * size of the object being freed, just in case it's needed.
 * On an 80x86 machine using small-data memory model, these manage near heap.
 */

extern "C" {
    pub fn jpeg_get_small(
        cinfo: j_common_ptr,
        sizeofobject: usize,
    ) -> *mut c_void;

    pub fn jpeg_free_small(
        cinfo: j_common_ptr,
        object: *mut c_void,
        sizeofobject: usize,
    );
}

/*
 * These two functions are used to allocate and release large chunks of
 * memory (up to the total free space designated by jpeg_mem_available).
 * The interface is the same as above, except that on an 80x86 machine,
 * far pointers are used.  On most other machines these are identical to
 * the jpeg_get/free_small routines; but we keep them separate anyway,
 * in case a different allocation strategy is desirable for large chunks.
 */

extern "C" {
    pub fn jpeg_get_large(
        cinfo: j_common_ptr,
        sizeofobject: usize,
    ) -> *mut c_void;

    pub fn jpeg_free_large(
        cinfo: j_common_ptr,
        object: *mut c_void,
        sizeofobject: usize,
    );
}

/*
 * The macro MAX_ALLOC_CHUNK designates the maximum number of bytes that may
 * be requested in a single call to jpeg_get_large (and jpeg_get_small for that
 * matter, but that case should never come into play).  This macro is needed
 * to model the 64Kb-segment-size limit of far addressing on 80x86 machines.
 * On those machines, we expect that jconfig.h will provide a proper value.
 * On machines with 32-bit flat address spaces, any large constant may be used.
 *
 * NB: jmemmgr.c expects that MAX_ALLOC_CHUNK will be representable as type
 * size_t and will be a multiple of sizeof(align_type).
 */

pub const MAX_ALLOC_CHUNK: usize = 1000000000;

/* may be overridden in jconfig.h */

/*
 * This routine computes the total space still available for allocation by
 * jpeg_get_large.  If more space than this is needed, backing store will be
 * used.  NOTE: any memory already allocated must not be counted.
 *
 * There is a minimum space requirement, corresponding to the minimum
 * feasible buffer sizes; jmemmgr.c will request that much space even if
 * jpeg_mem_available returns zero.  The maximum space needed, enough to hold
 * all working storage in memory, is also passed in case it is useful.
 * Finally, the total space already allocated is passed.  If no better
 * method is available, cinfo->mem->max_memory_to_use - already_allocated
 * is often a suitable calculation.
 *
 * It is OK for jpeg_mem_available to underestimate the space available
 * (that'll just lead to more backing-store access than is really necessary).
 * However, an overestimate will lead to failure.  Hence it's wise to subtract
 * a slop factor from the true available space.  5% should be enough.
 *
 * On machines with lots of virtual memory, any large constant may be returned.
 * Conversely, zero may be returned to always use the minimum amount of memory.
 */

extern "C" {
    pub fn jpeg_mem_available(
        cinfo: j_common_ptr,
        min_bytes_needed: i32,
        max_bytes_needed: i32,
        already_allocated: i32,
    ) -> i32;
}

/*
 * This structure holds whatever state is needed to access a single
 * backing-store object.  The read/write/close method pointers are called
 * by jmemmgr.c to manipulate the backing-store object; all other fields
 * are private to the system-dependent backing store routines.
 */

pub const TEMP_NAME_LENGTH: usize = 64; /* max length of a temporary file's name */

#[cfg(feature = "USE_MSDOS_MEMMGR")]
pub mod dos_memmgr {
    /* DOS-specific junk */

    pub type XMSH = u16; /* type of extended-memory handles */
    pub type EMSH = u16; /* type of expanded-memory handles */

    #[repr(C)]
    pub union handle_union {
        pub file_handle: i16, /* DOS file handle if it's a temp file */
        pub xms_handle: XMSH, /* handle if it's a chunk of XMS */
        pub ems_handle: EMSH, /* handle if it's a chunk of EMS */
    }
}

pub type backing_store_ptr = *mut backing_store_struct;

#[repr(C)]
pub struct backing_store_struct {
    /* Methods for reading/writing/closing this backing-store object */
    pub read_backing_store: extern "C" fn(
        cinfo: j_common_ptr,
        info: backing_store_ptr,
        buffer_address: *mut c_void,
        file_offset: i32,
        byte_count: i32,
    ),
    pub write_backing_store: extern "C" fn(
        cinfo: j_common_ptr,
        info: backing_store_ptr,
        buffer_address: *mut c_void,
        file_offset: i32,
        byte_count: i32,
    ),
    pub close_backing_store: extern "C" fn(
        cinfo: j_common_ptr,
        info: backing_store_ptr,
    ),

    /* Private fields for system-dependent backing-store management */
    #[cfg(feature = "USE_MSDOS_MEMMGR")]
    pub handle: dos_memmgr::handle_union, /* reference to backing-store storage object */
    #[cfg(feature = "USE_MSDOS_MEMMGR")]
    pub temp_name: [c_char; TEMP_NAME_LENGTH], /* name if it's a file */

    #[cfg(not(feature = "USE_MSDOS_MEMMGR"))]
    pub temp_file: *mut c_void, /* stdio reference to temp file */
    #[cfg(not(feature = "USE_MSDOS_MEMMGR"))]
    pub temp_name: [c_char; TEMP_NAME_LENGTH], /* name of temp file */
}

/*
 * Initial opening of a backing-store object.  This must fill in the
 * read/write/close pointers in the object.  The read/write routines
 * may take an error exit if the specified maximum file size is exceeded.
 * (If jpeg_mem_available always returns a large value, this routine can
 * just take an error exit.)
 */

extern "C" {
    pub fn jpeg_open_backing_store(
        cinfo: j_common_ptr,
        info: backing_store_ptr,
        total_bytes_needed: i32,
    );
}

/*
 * These routines take care of any system-dependent initialization and
 * cleanup required.  jpeg_mem_init will be called before anything is
 * allocated (and, therefore, nothing in cinfo is of use except the error
 * manager pointer).  It should return a suitable default value for
 * max_memory_to_use; this may subsequently be overridden by the surrounding
 * application.  (Note that max_memory_to_use is only important if
 * jpeg_mem_available chooses to consult it ... no one else will.)
 * jpeg_mem_term may assume that all requested memory has been freed and that
 * all opened backing-store objects have been closed.
 */

extern "C" {
    pub fn jpeg_mem_init(cinfo: j_common_ptr) -> i32;
    pub fn jpeg_mem_term(cinfo: j_common_ptr);
}
