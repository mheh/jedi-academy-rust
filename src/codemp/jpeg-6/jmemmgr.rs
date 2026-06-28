/*
 * jmemmgr.c
 *
 * Copyright (C) 1991-1995, Thomas G. Lane.
 * This file is part of the Independent JPEG Group's software.
 * For conditions of distribution and use, see the accompanying README file.
 *
 * This file contains the JPEG system-independent memory management
 * routines.  This code is usable across a wide variety of machines; most
 * of the system dependencies have been isolated in a separate file.
 * The major functions provided here are:
 *   * pool-based allocation and freeing of memory;
 *   * policy decisions about how to divide available memory among the
 *     virtual arrays;
 *   * control logic for swapping virtual arrays between main memory and
 *     backing storage.
 * The separate system-dependent file provides the actual backing-storage
 * access code, and it contains the policy decision about how much total
 * main memory to use.
 * This file is system-dependent in the sense that some of its functions
 * are unnecessary in some systems.  For example, if there is enough virtual
 * memory so that backing storage will never be used, much of the virtual
 * array control logic could be removed.  (Of course, if you have that much
 * memory then you shouldn't care about a little bit of unused code...)
 */

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};
use core::mem::size_of;

use super::jmemsys_h::{
    backing_store_info, j_common_ptr, jpeg_free_large, jpeg_free_small, jpeg_get_large,
    jpeg_get_small, jpeg_mem_available, jpeg_mem_init, jpeg_mem_term, jpeg_open_backing_store,
    MAX_ALLOC_CHUNK,
};

// Opaque type stubs for JPEG library types
pub type JSAMPLE = u8;
pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;
pub type JCOEF = i16;
pub type JBLOCK = [i16; 64];
pub type JBLOCKROW = *mut JBLOCK;
pub type JBLOCKARRAY = *mut JBLOCKROW;
pub type JDIMENSION = u32;
pub type boolean = u8;

pub struct jvirt_sarray_control;
pub struct jvirt_barray_control;

pub type jvirt_sarray_ptr = *mut jvirt_sarray_control;
pub type jvirt_barray_ptr = *mut jvirt_barray_control;

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    pub alloc_large: Option<unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void>,
    pub alloc_sarray:
        Option<unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> JSAMPARRAY>,
    pub alloc_barray:
        Option<unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> JBLOCKARRAY>,
    pub request_virt_sarray: Option<
        unsafe extern "C" fn(j_common_ptr, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION)
            -> jvirt_sarray_ptr,
    >,
    pub request_virt_barray: Option<
        unsafe extern "C" fn(j_common_ptr, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION)
            -> jvirt_barray_ptr,
    >,
    pub realize_virt_arrays: Option<unsafe extern "C" fn(j_common_ptr)>,
    pub access_virt_sarray: Option<
        unsafe extern "C" fn(j_common_ptr, jvirt_sarray_ptr, JDIMENSION, JDIMENSION, boolean)
            -> JSAMPARRAY,
    >,
    pub access_virt_barray: Option<
        unsafe extern "C" fn(j_common_ptr, jvirt_barray_ptr, JDIMENSION, JDIMENSION, boolean)
            -> JBLOCKARRAY,
    >,
    pub free_pool: Option<unsafe extern "C" fn(j_common_ptr, c_int)>,
    pub self_destruct: Option<unsafe extern "C" fn(j_common_ptr)>,
    pub max_memory_to_use: c_long,
}

/*
 * Some important notes:
 *   The allocation routines provided here must never return NULL.
 *   They should exit to error_exit if unsuccessful.
 *
 *   It's not a good idea to try to merge the sarray and barray routines,
 *   even though they are textually almost the same, because samples are
 *   usually stored as bytes while coefficients are shorts or ints.  Thus,
 *   in machines where byte pointers have a different representation from
 *   word pointers, the resulting machine code could not be the same.
 */

/*
 * Many machines require storage alignment: longs must start on 4-byte
 * boundaries, doubles on 8-byte boundaries, etc.  On such machines, malloc()
 * always returns pointers that are multiples of the worst-case alignment
 * requirement, and we had better do so too.
 * There isn't any really portable way to determine the worst-case alignment
 * requirement.  This module assumes that the alignment requirement is
 * multiples of sizeof(ALIGN_TYPE).
 * By default, we define ALIGN_TYPE as double.  This is necessary on some
 * workstations (where doubles really do need 8-byte alignment) and will work
 * fine on nearly everything.  If your machine has lesser alignment needs,
 * you can save a few bytes by making ALIGN_TYPE smaller.
 * The only place I know of where this will NOT work is certain Macintosh
 * 680x0 compilers that define double as a 10-byte IEEE extended float.
 * Doing 10-byte alignment is counterproductive because longwords won't be
 * aligned well.  Put "#define ALIGN_TYPE long" in jconfig.h if you have
 * such a compiler.
 */

type ALIGN_TYPE = f64;

#[inline]
fn SIZEOF<T>() -> usize {
    size_of::<T>()
}

/*
 * We allocate objects from "pools", where each pool is gotten with a single
 * request to jpeg_get_small() or jpeg_get_large().  There is no per-object
 * overhead within a pool, except for alignment padding.  Each pool has a
 * header with a link to the next pool of the same class.
 * Small and large pool headers are identical except that the latter's
 * link pointer must be FAR on 80x86 machines.
 * Notice that the "real" header fields are union'ed with a dummy ALIGN_TYPE
 * field.  This forces the compiler to make SIZEOF(small_pool_hdr) a multiple
 * of the alignment requirement of ALIGN_TYPE.
 */

type small_pool_ptr = *mut small_pool_struct;

#[repr(C)]
union small_pool_struct {
    hdr: small_pool_hdr_fields,
    dummy: ALIGN_TYPE,
}

#[repr(C)]
struct small_pool_hdr_fields {
    next: small_pool_ptr,
    bytes_used: usize,
    bytes_left: usize,
}

type large_pool_ptr = *mut large_pool_struct;

#[repr(C)]
union large_pool_struct {
    hdr: large_pool_hdr_fields,
    dummy: ALIGN_TYPE,
}

#[repr(C)]
struct large_pool_hdr_fields {
    next: large_pool_ptr,
    bytes_used: usize,
    bytes_left: usize,
}

/*
 * Here is the full definition of a memory manager object.
 */

#[repr(C)]
struct my_memory_mgr {
    pub_: jpeg_memory_mgr,

    /* Each pool identifier (lifetime class) names a linked list of pools. */
    small_list: [small_pool_ptr; 2],
    large_list: [large_pool_ptr; 2],

    /* Since we only have one lifetime class of virtual arrays, only one
     * linked list is necessary (for each datatype).  Note that the virtual
     * array control blocks being linked together are actually stored somewhere
     * in the small-pool list.
     */
    virt_sarray_list: jvirt_sarray_ptr,
    virt_barray_list: jvirt_barray_ptr,

    /* This counts total space obtained from jpeg_get_small/large */
    total_space_allocated: c_long,

    /* alloc_sarray and alloc_barray set this value for use by virtual
     * array routines.
     */
    last_rowsperchunk: JDIMENSION,
}

type my_mem_ptr = *mut my_memory_mgr;

/*
 * The control blocks for virtual arrays.
 * Note that these blocks are allocated in the "small" pool area.
 * System-dependent info for the associated backing store (if any) is hidden
 * inside the backing_store_info struct.
 */

pub struct jvirt_sarray_control {
    mem_buffer: JSAMPARRAY,
    rows_in_array: JDIMENSION,
    samplesperrow: JDIMENSION,
    maxaccess: JDIMENSION,
    rows_in_mem: JDIMENSION,
    rowsperchunk: JDIMENSION,
    cur_start_row: JDIMENSION,
    first_undef_row: JDIMENSION,
    pre_zero: boolean,
    dirty: boolean,
    b_s_open: boolean,
    next: jvirt_sarray_ptr,
    b_s_info: backing_store_info,
}

pub struct jvirt_barray_control {
    mem_buffer: JBLOCKARRAY,
    rows_in_array: JDIMENSION,
    blocksperrow: JDIMENSION,
    maxaccess: JDIMENSION,
    rows_in_mem: JDIMENSION,
    rowsperchunk: JDIMENSION,
    cur_start_row: JDIMENSION,
    first_undef_row: JDIMENSION,
    pre_zero: boolean,
    dirty: boolean,
    b_s_open: boolean,
    next: jvirt_barray_ptr,
    b_s_info: backing_store_info,
}

/* Constants for pool lifetime classes */
const JPOOL_PERMANENT: c_int = 0;
const JPOOL_IMAGE: c_int = 1;
const JPOOL_NUMPOOLS: c_int = 2;

#[cfg(feature = "MEM_STATS")]
unsafe fn print_mem_stats(cinfo: j_common_ptr, pool_id: c_int) {
    let mem = (*cinfo).mem as my_mem_ptr;
    let mut shdr_ptr: small_pool_ptr;
    let mut lhdr_ptr: large_pool_ptr;

    /* Since this is only a debugging stub, we can cheat a little by using
     * fprintf directly rather than going through the trace message code.
     * This is helpful because message parm array can't handle longs.
     */
    extern "C" {
        fn fprintf(stream: *mut c_void, format: *const c_char, ...) -> c_int;
    }

    fprintf(
        core::ptr::null_mut(),
        b"Freeing pool %d, total space = %ld\n\0".as_ptr() as *const c_char,
        pool_id,
        (*mem).total_space_allocated,
    );

    lhdr_ptr = (*mem).large_list[pool_id as usize];
    while !lhdr_ptr.is_null() {
        fprintf(
            core::ptr::null_mut(),
            b"  Large chunk used %ld\n\0".as_ptr() as *const c_char,
            (*lhdr_ptr).hdr.bytes_used as c_long,
        );
        lhdr_ptr = (*lhdr_ptr).hdr.next;
    }

    shdr_ptr = (*mem).small_list[pool_id as usize];
    while !shdr_ptr.is_null() {
        fprintf(
            core::ptr::null_mut(),
            b"  Small chunk used %ld free %ld\n\0".as_ptr() as *const c_char,
            (*shdr_ptr).hdr.bytes_used as c_long,
            (*shdr_ptr).hdr.bytes_left as c_long,
        );
        shdr_ptr = (*shdr_ptr).hdr.next;
    }
}

extern "C" {
    fn ERREXIT1(cinfo: j_common_ptr, msg_code: c_int, msg_parm: c_int);
    fn ERREXIT(cinfo: j_common_ptr, msg_code: c_int);
}

const JERR_OUT_OF_MEMORY: c_int = 1;
const JERR_BAD_POOL_ID: c_int = 2;
const JERR_WIDTH_OVERFLOW: c_int = 3;
const JERR_BAD_ALIGN_TYPE: c_int = 4;
const JERR_BAD_ALLOC_CHUNK: c_int = 5;
const JERR_BAD_VIRTUAL_ACCESS: c_int = 6;
const JERR_VIRTUAL_BUG: c_int = 7;

unsafe fn out_of_memory(cinfo: j_common_ptr, which: c_int) {
    /* Report an out-of-memory error and stop execution */
    /* If we compiled MEM_STATS support, report alloc requests before dying */
    #[cfg(feature = "MEM_STATS")]
    {
        (*(*cinfo).err).trace_level = 2;
    }
    ERREXIT1(cinfo, JERR_OUT_OF_MEMORY, which);
}

/*
 * Allocation of "small" objects.
 *
 * For these, we use pooled storage.  When a new pool must be created,
 * we try to get enough space for the current request plus a "slop" factor,
 * where the slop will be the amount of leftover space in the new pool.
 * The speed vs. space tradeoff is largely determined by the slop values.
 * A different slop value is provided for each pool class (lifetime),
 * and we also distinguish the first pool of a class from later ones.
 * NOTE: the values given work fairly well on both 16- and 32-bit-int
 * machines, but may be too small if longs are 64 bits or more.
 */

static FIRST_POOL_SLOP: [usize; 2] = [
    1600,  /* first PERMANENT pool */
    16000, /* first IMAGE pool */
];

static EXTRA_POOL_SLOP: [usize; 2] = [
    0,    /* additional PERMANENT pools */
    5000, /* additional IMAGE pools */
];

const MIN_SLOP: usize = 50;

unsafe fn alloc_small(cinfo: j_common_ptr, pool_id: c_int, sizeofobject: usize) -> *mut c_void {
    /* Allocate a "small" object */
    let mem = (*cinfo).mem as my_mem_ptr;
    let mut hdr_ptr: small_pool_ptr;
    let mut prev_hdr_ptr: small_pool_ptr;
    let mut data_ptr: *mut c_char;
    let mut odd_bytes: usize;
    let mut min_request: usize;
    let mut slop: usize;

    /* Check for unsatisfiable request (do now to ensure no overflow below) */
    if sizeofobject > (MAX_ALLOC_CHUNK as usize).wrapping_sub(SIZEOF::<small_pool_struct>()) {
        out_of_memory(cinfo, 1); /* request exceeds malloc's ability */
    }

    /* Round up the requested size to a multiple of SIZEOF(ALIGN_TYPE) */
    odd_bytes = sizeofobject % SIZEOF::<ALIGN_TYPE>();
    if odd_bytes > 0 {
        let adjusted_size = sizeofobject.wrapping_add(SIZEOF::<ALIGN_TYPE>().wrapping_sub(odd_bytes));
        let sizeofobject_mut = adjusted_size;
        // Since we can't reassign in Rust easily, use a new variable
        let mut adjusted = sizeofobject;
        adjusted = adjusted.wrapping_add(SIZEOF::<ALIGN_TYPE>().wrapping_sub(odd_bytes));
    }
    let mut sizeofobject = sizeofobject;
    if odd_bytes > 0 {
        sizeofobject = sizeofobject.wrapping_add(SIZEOF::<ALIGN_TYPE>().wrapping_sub(odd_bytes));
    }

    /* See if space is available in any existing pool */
    if pool_id < 0 || pool_id >= JPOOL_NUMPOOLS {
        ERREXIT1(cinfo, JERR_BAD_POOL_ID, pool_id); /* safety check */
    }
    prev_hdr_ptr = core::ptr::null_mut();
    hdr_ptr = (*mem).small_list[pool_id as usize];
    while !hdr_ptr.is_null() {
        if (*hdr_ptr).hdr.bytes_left >= sizeofobject {
            break; /* found pool with enough space */
        }
        prev_hdr_ptr = hdr_ptr;
        hdr_ptr = (*hdr_ptr).hdr.next;
    }

    /* Time to make a new pool? */
    if hdr_ptr.is_null() {
        /* min_request is what we need now, slop is what will be leftover */
        min_request = sizeofobject.wrapping_add(SIZEOF::<small_pool_struct>());
        if prev_hdr_ptr.is_null() {
            /* first pool in class? */
            slop = FIRST_POOL_SLOP[pool_id as usize];
        } else {
            slop = EXTRA_POOL_SLOP[pool_id as usize];
        }
        /* Don't ask for more than MAX_ALLOC_CHUNK */
        if slop > (MAX_ALLOC_CHUNK as usize).wrapping_sub(min_request) {
            slop = (MAX_ALLOC_CHUNK as usize).wrapping_sub(min_request);
        }
        /* Try to get space, if fail reduce slop and try again */
        loop {
            hdr_ptr = jpeg_get_small(cinfo, min_request.wrapping_add(slop)) as small_pool_ptr;
            if !hdr_ptr.is_null() {
                break;
            }
            slop /= 2;
            if slop < MIN_SLOP {
                /* give up when it gets real small */
                out_of_memory(cinfo, 2); /* jpeg_get_small failed */
            }
        }
        (*mem).total_space_allocated =
            (*mem).total_space_allocated.wrapping_add((min_request.wrapping_add(slop)) as c_long);
        /* Success, initialize the new pool header and add to end of list */
        (*hdr_ptr).hdr.next = core::ptr::null_mut();
        (*hdr_ptr).hdr.bytes_used = 0;
        (*hdr_ptr).hdr.bytes_left = sizeofobject.wrapping_add(slop);
        if prev_hdr_ptr.is_null() {
            /* first pool in class? */
            (*mem).small_list[pool_id as usize] = hdr_ptr;
        } else {
            (*prev_hdr_ptr).hdr.next = hdr_ptr;
        }
    }

    /* OK, allocate the object from the current pool */
    data_ptr = (hdr_ptr as *mut c_char).wrapping_add(1); /* point to first data byte in pool */
    data_ptr = data_ptr.wrapping_add((*hdr_ptr).hdr.bytes_used); /* point to place for object */
    (*hdr_ptr).hdr.bytes_used = (*hdr_ptr).hdr.bytes_used.wrapping_add(sizeofobject);
    (*hdr_ptr).hdr.bytes_left = (*hdr_ptr).hdr.bytes_left.wrapping_sub(sizeofobject);

    data_ptr as *mut c_void
}

/*
 * Allocation of "large" objects.
 *
 * The external semantics of these are the same as "small" objects,
 * except that FAR pointers are used on 80x86.  However the pool
 * management heuristics are quite different.  We assume that each
 * request is large enough that it may as well be passed directly to
 * jpeg_get_large; the pool management just links everything together
 * so that we can free it all on demand.
 * Note: the major use of "large" objects is in JSAMPARRAY and JBLOCKARRAY
 * structures.  The routines that create these structures (see below)
 * deliberately bunch rows together to ensure a large request size.
 */

unsafe fn alloc_large(cinfo: j_common_ptr, pool_id: c_int, sizeofobject: usize) -> *mut c_void {
    /* Allocate a "large" object */
    let mem = (*cinfo).mem as my_mem_ptr;
    let mut hdr_ptr: large_pool_ptr;
    let mut odd_bytes: usize;

    /* Check for unsatisfiable request (do now to ensure no overflow below) */
    if sizeofobject > (MAX_ALLOC_CHUNK as usize).wrapping_sub(SIZEOF::<large_pool_struct>()) {
        out_of_memory(cinfo, 3); /* request exceeds malloc's ability */
    }

    /* Round up the requested size to a multiple of SIZEOF(ALIGN_TYPE) */
    odd_bytes = sizeofobject % SIZEOF::<ALIGN_TYPE>();
    if odd_bytes > 0 {
        let mut sizeofobject = sizeofobject;
        sizeofobject = sizeofobject.wrapping_add(SIZEOF::<ALIGN_TYPE>().wrapping_sub(odd_bytes));
    }
    let mut sizeofobject = sizeofobject;
    if odd_bytes > 0 {
        sizeofobject = sizeofobject.wrapping_add(SIZEOF::<ALIGN_TYPE>().wrapping_sub(odd_bytes));
    }

    /* Always make a new pool */
    if pool_id < 0 || pool_id >= JPOOL_NUMPOOLS {
        ERREXIT1(cinfo, JERR_BAD_POOL_ID, pool_id); /* safety check */
    }

    hdr_ptr = jpeg_get_large(cinfo, sizeofobject.wrapping_add(SIZEOF::<large_pool_struct>())) as large_pool_ptr;
    if hdr_ptr.is_null() {
        out_of_memory(cinfo, 4); /* jpeg_get_large failed */
    }
    (*mem).total_space_allocated = (*mem).total_space_allocated.wrapping_add(
        (sizeofobject.wrapping_add(SIZEOF::<large_pool_struct>())) as c_long,
    );

    /* Success, initialize the new pool header and add to list */
    (*hdr_ptr).hdr.next = (*mem).large_list[pool_id as usize];
    /* We maintain space counts in each pool header for statistical purposes,
     * even though they are not needed for allocation.
     */
    (*hdr_ptr).hdr.bytes_used = sizeofobject;
    (*hdr_ptr).hdr.bytes_left = 0;
    (*mem).large_list[pool_id as usize] = hdr_ptr;

    (hdr_ptr as *mut c_char).wrapping_add(1) as *mut c_void /* point to first data byte in pool */
}

/*
 * Creation of 2-D sample arrays.
 * The pointers are in near heap, the samples themselves in FAR heap.
 *
 * To minimize allocation overhead and to allow I/O of large contiguous
 * blocks, we allocate the sample rows in groups of as many rows as possible
 * without exceeding MAX_ALLOC_CHUNK total bytes per allocation request.
 * NB: the virtual array control routines, later in this file, know about
 * this chunking of rows.  The rowsperchunk value is left in the mem manager
 * object so that it can be saved away if this sarray is the workspace for
 * a virtual array.
 */

unsafe fn alloc_sarray(
    cinfo: j_common_ptr,
    pool_id: c_int,
    samplesperrow: JDIMENSION,
    numrows: JDIMENSION,
) -> JSAMPARRAY {
    /* Allocate a 2-D sample array */
    let mem = (*cinfo).mem as my_mem_ptr;
    let result: JSAMPARRAY;
    let mut workspace: JSAMPROW;
    let mut rowsperchunk: JDIMENSION;
    let mut currow: JDIMENSION;
    let mut i: JDIMENSION;
    let mut ltemp: c_long;

    /* Calculate max # of rows allowed in one allocation chunk */
    ltemp = ((MAX_ALLOC_CHUNK - SIZEOF::<large_pool_struct>() as i64) as c_long)
        / ((samplesperrow as c_long) * (SIZEOF::<JSAMPLE>() as c_long));
    if ltemp <= 0 {
        ERREXIT(cinfo, JERR_WIDTH_OVERFLOW);
    }
    if ltemp < (numrows as c_long) {
        rowsperchunk = ltemp as JDIMENSION;
    } else {
        rowsperchunk = numrows;
    }
    (*mem).last_rowsperchunk = rowsperchunk;

    /* Get space for row pointers (small object) */
    result = alloc_small(cinfo, pool_id, (numrows as usize) * SIZEOF::<JSAMPROW>()) as JSAMPARRAY;

    /* Get the rows themselves (large objects) */
    currow = 0;
    while currow < numrows {
        rowsperchunk = if rowsperchunk < numrows - currow {
            rowsperchunk
        } else {
            numrows - currow
        };
        workspace = alloc_large(
            cinfo,
            pool_id,
            ((rowsperchunk as usize) * (samplesperrow as usize)) * SIZEOF::<JSAMPLE>(),
        ) as JSAMPROW;
        i = rowsperchunk;
        while i > 0 {
            *result.wrapping_add(currow as usize) = workspace;
            workspace = workspace.wrapping_add(samplesperrow as usize);
            currow = currow.wrapping_add(1);
            i = i.wrapping_sub(1);
        }
    }

    result
}

/*
 * Creation of 2-D coefficient-block arrays.
 * This is essentially the same as the code for sample arrays, above.
 */

unsafe fn alloc_barray(
    cinfo: j_common_ptr,
    pool_id: c_int,
    blocksperrow: JDIMENSION,
    numrows: JDIMENSION,
) -> JBLOCKARRAY {
    /* Allocate a 2-D coefficient-block array */
    let mem = (*cinfo).mem as my_mem_ptr;
    let result: JBLOCKARRAY;
    let mut workspace: JBLOCKROW;
    let mut rowsperchunk: JDIMENSION;
    let mut currow: JDIMENSION;
    let mut i: JDIMENSION;
    let mut ltemp: c_long;

    /* Calculate max # of rows allowed in one allocation chunk */
    ltemp = ((MAX_ALLOC_CHUNK - SIZEOF::<large_pool_struct>() as i64) as c_long)
        / ((blocksperrow as c_long) * (SIZEOF::<JBLOCK>() as c_long));
    if ltemp <= 0 {
        ERREXIT(cinfo, JERR_WIDTH_OVERFLOW);
    }
    if ltemp < (numrows as c_long) {
        rowsperchunk = ltemp as JDIMENSION;
    } else {
        rowsperchunk = numrows;
    }
    (*mem).last_rowsperchunk = rowsperchunk;

    /* Get space for row pointers (small object) */
    result = alloc_small(cinfo, pool_id, (numrows as usize) * SIZEOF::<JBLOCKROW>()) as JBLOCKARRAY;

    /* Get the rows themselves (large objects) */
    currow = 0;
    while currow < numrows {
        rowsperchunk = if rowsperchunk < numrows - currow {
            rowsperchunk
        } else {
            numrows - currow
        };
        workspace = alloc_large(
            cinfo,
            pool_id,
            ((rowsperchunk as usize) * (blocksperrow as usize)) * SIZEOF::<JBLOCK>(),
        ) as JBLOCKROW;
        i = rowsperchunk;
        while i > 0 {
            *result.wrapping_add(currow as usize) = workspace;
            workspace = workspace.wrapping_add(blocksperrow as usize);
            currow = currow.wrapping_add(1);
            i = i.wrapping_sub(1);
        }
    }

    result
}

/*
 * About virtual array management:
 *
 * The above "normal" array routines are only used to allocate strip buffers
 * (as wide as the image, but just a few rows high).  Full-image-sized buffers
 * are handled as "virtual" arrays.  The array is still accessed a strip at a
 * time, but the memory manager must save the whole array for repeated
 * accesses.  The intended implementation is that there is a strip buffer in
 * memory (as high as is possible given the desired memory limit), plus a
 * backing file that holds the rest of the array.
 *
 * The request_virt_array routines are told the total size of the image and
 * the maximum number of rows that will be accessed at once.  The in-memory
 * buffer must be at least as large as the maxaccess value.
 *
 * The request routines create control blocks but not the in-memory buffers.
 * That is postponed until realize_virt_arrays is called.  At that time the
 * total amount of space needed is known (approximately, anyway), so free
 * memory can be divided up fairly.
 *
 * The access_virt_array routines are responsible for making a specific strip
 * area accessible (after reading or writing the backing file, if necessary).
 * Note that the access routines are told whether the caller intends to modify
 * the accessed strip; during a read-only pass this saves having to rewrite
 * data to disk.  The access routines are also responsible for pre-zeroing
 * any newly accessed rows, if pre-zeroing was requested.
 *
 * In current usage, the access requests are usually for nonoverlapping
 * strips; that is, successive access start_row numbers differ by exactly
 * num_rows = maxaccess.  This means we can get good performance with simple
 * buffer dump/reload logic, by making the in-memory buffer be a multiple
 * of the access height; then there will never be accesses across bufferload
 * boundaries.  The code will still work with overlapping access requests,
 * but it doesn't handle bufferload overlaps very efficiently.
 */

unsafe fn request_virt_sarray(
    cinfo: j_common_ptr,
    pool_id: c_int,
    pre_zero: boolean,
    samplesperrow: JDIMENSION,
    numrows: JDIMENSION,
    maxaccess: JDIMENSION,
) -> jvirt_sarray_ptr {
    /* Request a virtual 2-D sample array */
    let mem = (*cinfo).mem as my_mem_ptr;
    let result: jvirt_sarray_ptr;

    /* Only IMAGE-lifetime virtual arrays are currently supported */
    if pool_id != JPOOL_IMAGE {
        ERREXIT1(cinfo, JERR_BAD_POOL_ID, pool_id); /* safety check */
    }

    /* get control block */
    result = alloc_small(
        cinfo,
        pool_id,
        SIZEOF::<jvirt_sarray_control>(),
    ) as jvirt_sarray_ptr;

    (*result).mem_buffer = core::ptr::null_mut(); /* marks array not yet realized */
    (*result).rows_in_array = numrows;
    (*result).samplesperrow = samplesperrow;
    (*result).maxaccess = maxaccess;
    (*result).pre_zero = pre_zero;
    (*result).b_s_open = 0; /* no associated backing-store object */
    (*result).next = (*mem).virt_sarray_list; /* add to list of virtual arrays */
    (*mem).virt_sarray_list = result;

    result
}

unsafe fn request_virt_barray(
    cinfo: j_common_ptr,
    pool_id: c_int,
    pre_zero: boolean,
    blocksperrow: JDIMENSION,
    numrows: JDIMENSION,
    maxaccess: JDIMENSION,
) -> jvirt_barray_ptr {
    /* Request a virtual 2-D coefficient-block array */
    let mem = (*cinfo).mem as my_mem_ptr;
    let result: jvirt_barray_ptr;

    /* Only IMAGE-lifetime virtual arrays are currently supported */
    if pool_id != JPOOL_IMAGE {
        ERREXIT1(cinfo, JERR_BAD_POOL_ID, pool_id); /* safety check */
    }

    /* get control block */
    result = alloc_small(
        cinfo,
        pool_id,
        SIZEOF::<jvirt_barray_control>(),
    ) as jvirt_barray_ptr;

    (*result).mem_buffer = core::ptr::null_mut(); /* marks array not yet realized */
    (*result).rows_in_array = numrows;
    (*result).blocksperrow = blocksperrow;
    (*result).maxaccess = maxaccess;
    (*result).pre_zero = pre_zero;
    (*result).b_s_open = 0; /* no associated backing-store object */
    (*result).next = (*mem).virt_barray_list; /* add to list of virtual arrays */
    (*mem).virt_barray_list = result;

    result
}

unsafe fn realize_virt_arrays(cinfo: j_common_ptr) {
    /* Allocate the in-memory buffers for any unrealized virtual arrays */
    let mem = (*cinfo).mem as my_mem_ptr;
    let mut space_per_minheight: c_long;
    let mut maximum_space: c_long;
    let mut avail_mem: c_long;
    let mut minheights: c_long;
    let mut max_minheights: c_long;
    let mut sptr: jvirt_sarray_ptr;
    let mut bptr: jvirt_barray_ptr;

    /* Compute the minimum space needed (maxaccess rows in each buffer)
     * and the maximum space needed (full image height in each buffer).
     * These may be of use to the system-dependent jpeg_mem_available routine.
     */
    space_per_minheight = 0;
    maximum_space = 0;
    sptr = (*mem).virt_sarray_list;
    while !sptr.is_null() {
        if (*sptr).mem_buffer.is_null() {
            /* if not realized yet */
            space_per_minheight = space_per_minheight.wrapping_add(
                (((*sptr).maxaccess as c_long) * ((*sptr).samplesperrow as c_long))
                    * (SIZEOF::<JSAMPLE>() as c_long),
            );
            maximum_space = maximum_space.wrapping_add(
                (((*sptr).rows_in_array as c_long) * ((*sptr).samplesperrow as c_long))
                    * (SIZEOF::<JSAMPLE>() as c_long),
            );
        }
        sptr = (*sptr).next;
    }
    bptr = (*mem).virt_barray_list;
    while !bptr.is_null() {
        if (*bptr).mem_buffer.is_null() {
            /* if not realized yet */
            space_per_minheight = space_per_minheight.wrapping_add(
                (((*bptr).maxaccess as c_long) * ((*bptr).blocksperrow as c_long))
                    * (SIZEOF::<JBLOCK>() as c_long),
            );
            maximum_space = maximum_space.wrapping_add(
                (((*bptr).rows_in_array as c_long) * ((*bptr).blocksperrow as c_long))
                    * (SIZEOF::<JBLOCK>() as c_long),
            );
        }
        bptr = (*bptr).next;
    }

    if space_per_minheight <= 0 {
        return; /* no unrealized arrays, no work */
    }

    /* Determine amount of memory to actually use; this is system-dependent. */
    avail_mem = jpeg_mem_available(cinfo, space_per_minheight, maximum_space, (*mem).total_space_allocated);

    /* If the maximum space needed is available, make all the buffers full
     * height; otherwise parcel it out with the same number of minheights
     * in each buffer.
     */
    if avail_mem >= maximum_space {
        max_minheights = 1000000000;
    } else {
        max_minheights = avail_mem / space_per_minheight;
        /* If there doesn't seem to be enough space, try to get the minimum
         * anyway.  This allows a "stub" implementation of jpeg_mem_available().
         */
        if max_minheights <= 0 {
            max_minheights = 1;
        }
    }

    /* Allocate the in-memory buffers and initialize backing store as needed. */

    sptr = (*mem).virt_sarray_list;
    while !sptr.is_null() {
        if (*sptr).mem_buffer.is_null() {
            /* if not realized yet */
            minheights = (((*sptr).rows_in_array as c_long) - 1) / ((*sptr).maxaccess as c_long) + 1;
            if minheights <= max_minheights {
                /* This buffer fits in memory */
                (*sptr).rows_in_mem = (*sptr).rows_in_array;
            } else {
                /* It doesn't fit in memory, create backing store. */
                (*sptr).rows_in_mem = (max_minheights * ((*sptr).maxaccess as c_long)) as JDIMENSION;
                jpeg_open_backing_store(
                    cinfo,
                    &mut (*sptr).b_s_info,
                    ((*sptr).rows_in_array as c_long)
                        * ((*sptr).samplesperrow as c_long)
                        * (SIZEOF::<JSAMPLE>() as c_long),
                );
                (*sptr).b_s_open = 1;
            }
            (*sptr).mem_buffer = alloc_sarray(cinfo, JPOOL_IMAGE, (*sptr).samplesperrow, (*sptr).rows_in_mem);
            (*sptr).rowsperchunk = (*mem).last_rowsperchunk;
            (*sptr).cur_start_row = 0;
            (*sptr).first_undef_row = 0;
            (*sptr).dirty = 0;
        }
        sptr = (*sptr).next;
    }

    bptr = (*mem).virt_barray_list;
    while !bptr.is_null() {
        if (*bptr).mem_buffer.is_null() {
            /* if not realized yet */
            minheights = (((*bptr).rows_in_array as c_long) - 1) / ((*bptr).maxaccess as c_long) + 1;
            if minheights <= max_minheights {
                /* This buffer fits in memory */
                (*bptr).rows_in_mem = (*bptr).rows_in_array;
            } else {
                /* It doesn't fit in memory, create backing store. */
                (*bptr).rows_in_mem = (max_minheights * ((*bptr).maxaccess as c_long)) as JDIMENSION;
                jpeg_open_backing_store(
                    cinfo,
                    &mut (*bptr).b_s_info,
                    ((*bptr).rows_in_array as c_long)
                        * ((*bptr).blocksperrow as c_long)
                        * (SIZEOF::<JBLOCK>() as c_long),
                );
                (*bptr).b_s_open = 1;
            }
            (*bptr).mem_buffer = alloc_barray(cinfo, JPOOL_IMAGE, (*bptr).blocksperrow, (*bptr).rows_in_mem);
            (*bptr).rowsperchunk = (*mem).last_rowsperchunk;
            (*bptr).cur_start_row = 0;
            (*bptr).first_undef_row = 0;
            (*bptr).dirty = 0;
        }
        bptr = (*bptr).next;
    }
}

unsafe fn do_sarray_io(cinfo: j_common_ptr, ptr: jvirt_sarray_ptr, writing: boolean) {
    /* Do backing store read or write of a virtual sample array */
    let mut bytesperrow: c_long;
    let mut file_offset: c_long;
    let mut byte_count: c_long;
    let mut rows: c_long;
    let mut thisrow: c_long;
    let mut i: c_long;

    bytesperrow =
        ((*ptr).samplesperrow as c_long) * (SIZEOF::<JSAMPLE>() as c_long);
    file_offset = ((*ptr).cur_start_row as c_long) * bytesperrow;
    /* Loop to read or write each allocation chunk in mem_buffer */
    i = 0;
    while i < ((*ptr).rows_in_mem as c_long) {
        /* One chunk, but check for short chunk at end of buffer */
        rows = if ((*ptr).rowsperchunk as c_long) < ((*ptr).rows_in_mem as c_long) - i {
            (*ptr).rowsperchunk as c_long
        } else {
            ((*ptr).rows_in_mem as c_long) - i
        };
        /* Transfer no more than is currently defined */
        thisrow = ((*ptr).cur_start_row as c_long) + i;
        rows = if rows < ((*ptr).first_undef_row as c_long) - thisrow {
            rows
        } else {
            ((*ptr).first_undef_row as c_long) - thisrow
        };
        /* Transfer no more than fits in file */
        rows = if rows < ((*ptr).rows_in_array as c_long) - thisrow {
            rows
        } else {
            ((*ptr).rows_in_array as c_long) - thisrow
        };
        if rows <= 0 {
            /* this chunk might be past end of file! */
            break;
        }
        byte_count = rows * bytesperrow;
        if writing != 0 {
            ((*(*ptr).b_s_info.write_backing_store)(
                cinfo,
                &mut (*ptr).b_s_info,
                *(*ptr).mem_buffer.wrapping_add(i as usize) as *mut c_void,
                file_offset,
                byte_count,
            ))
        } else {
            ((*(*ptr).b_s_info.read_backing_store)(
                cinfo,
                &mut (*ptr).b_s_info,
                *(*ptr).mem_buffer.wrapping_add(i as usize) as *mut c_void,
                file_offset,
                byte_count,
            ))
        }
        file_offset = file_offset.wrapping_add(byte_count);
        i = i.wrapping_add(((*ptr).rowsperchunk as c_long));
    }
}

unsafe fn do_barray_io(cinfo: j_common_ptr, ptr: jvirt_barray_ptr, writing: boolean) {
    /* Do backing store read or write of a virtual coefficient-block array */
    let mut bytesperrow: c_long;
    let mut file_offset: c_long;
    let mut byte_count: c_long;
    let mut rows: c_long;
    let mut thisrow: c_long;
    let mut i: c_long;

    bytesperrow = ((*ptr).blocksperrow as c_long) * (SIZEOF::<JBLOCK>() as c_long);
    file_offset = ((*ptr).cur_start_row as c_long) * bytesperrow;
    /* Loop to read or write each allocation chunk in mem_buffer */
    i = 0;
    while i < ((*ptr).rows_in_mem as c_long) {
        /* One chunk, but check for short chunk at end of buffer */
        rows = if ((*ptr).rowsperchunk as c_long) < ((*ptr).rows_in_mem as c_long) - i {
            (*ptr).rowsperchunk as c_long
        } else {
            ((*ptr).rows_in_mem as c_long) - i
        };
        /* Transfer no more than is currently defined */
        thisrow = ((*ptr).cur_start_row as c_long) + i;
        rows = if rows < ((*ptr).first_undef_row as c_long) - thisrow {
            rows
        } else {
            ((*ptr).first_undef_row as c_long) - thisrow
        };
        /* Transfer no more than fits in file */
        rows = if rows < ((*ptr).rows_in_array as c_long) - thisrow {
            rows
        } else {
            ((*ptr).rows_in_array as c_long) - thisrow
        };
        if rows <= 0 {
            /* this chunk might be past end of file! */
            break;
        }
        byte_count = rows * bytesperrow;
        if writing != 0 {
            ((*(*ptr).b_s_info.write_backing_store)(
                cinfo,
                &mut (*ptr).b_s_info,
                *(*ptr).mem_buffer.wrapping_add(i as usize) as *mut c_void,
                file_offset,
                byte_count,
            ))
        } else {
            ((*(*ptr).b_s_info.read_backing_store)(
                cinfo,
                &mut (*ptr).b_s_info,
                *(*ptr).mem_buffer.wrapping_add(i as usize) as *mut c_void,
                file_offset,
                byte_count,
            ))
        }
        file_offset = file_offset.wrapping_add(byte_count);
        i = i.wrapping_add(((*ptr).rowsperchunk as c_long));
    }
}

unsafe fn access_virt_sarray(
    cinfo: j_common_ptr,
    ptr: jvirt_sarray_ptr,
    start_row: JDIMENSION,
    num_rows: JDIMENSION,
    writable: boolean,
) -> JSAMPARRAY {
    /* Access the part of a virtual sample array starting at start_row */
    /* and extending for num_rows rows.  writable is true if  */
    /* caller intends to modify the accessed area. */
    let end_row: JDIMENSION = start_row.wrapping_add(num_rows);
    let mut undef_row: JDIMENSION;

    /* debugging check */
    if end_row > (*ptr).rows_in_array
        || num_rows > (*ptr).maxaccess
        || (*ptr).mem_buffer.is_null()
    {
        ERREXIT(cinfo, JERR_BAD_VIRTUAL_ACCESS);
    }

    /* Make the desired part of the virtual array accessible */
    if start_row < (*ptr).cur_start_row || end_row > (*ptr).cur_start_row.wrapping_add((*ptr).rows_in_mem) {
        if (*ptr).b_s_open == 0 {
            ERREXIT(cinfo, JERR_VIRTUAL_BUG);
        }
        /* Flush old buffer contents if necessary */
        if (*ptr).dirty != 0 {
            do_sarray_io(cinfo, ptr, 1);
            (*ptr).dirty = 0;
        }
        /* Decide what part of virtual array to access.
         * Algorithm: if target address > current window, assume forward scan,
         * load starting at target address.  If target address < current window,
         * assume backward scan, load so that target area is top of window.
         * Note that when switching from forward write to forward read, will have
         * start_row = 0, so the limiting case applies and we load from 0 anyway.
         */
        if start_row > (*ptr).cur_start_row {
            (*ptr).cur_start_row = start_row;
        } else {
            /* use long arithmetic here to avoid overflow & unsigned problems */
            let mut ltemp: c_long;

            ltemp = (end_row as c_long) - ((*ptr).rows_in_mem as c_long);
            if ltemp < 0 {
                ltemp = 0; /* don't fall off front end of file */
            }
            (*ptr).cur_start_row = ltemp as JDIMENSION;
        }
        /* Read in the selected part of the array.
         * During the initial write pass, we will do no actual read
         * because the selected part is all undefined.
         */
        do_sarray_io(cinfo, ptr, 0);
    }
    /* Ensure the accessed part of the array is defined; prezero if needed.
     * To improve locality of access, we only prezero the part of the array
     * that the caller is about to access, not the entire in-memory array.
     */
    if (*ptr).first_undef_row < end_row {
        if (*ptr).first_undef_row < start_row {
            if writable != 0 {
                /* writer skipped over a section of array */
                ERREXIT(cinfo, JERR_BAD_VIRTUAL_ACCESS);
            }
            undef_row = start_row; /* but reader is allowed to read ahead */
        } else {
            undef_row = (*ptr).first_undef_row;
        }
        if writable != 0 {
            (*ptr).first_undef_row = end_row;
        }
        if (*ptr).pre_zero != 0 {
            let bytesperrow: usize = ((*ptr).samplesperrow as usize) * SIZEOF::<JSAMPLE>();
            undef_row = undef_row.wrapping_sub((*ptr).cur_start_row); /* make indexes relative to buffer */
            let mut end_row_adjusted = end_row.wrapping_sub((*ptr).cur_start_row);
            while undef_row < end_row_adjusted {
                jzero_far(
                    *(*ptr).mem_buffer.wrapping_add(undef_row as usize) as *mut c_void,
                    bytesperrow,
                );
                undef_row = undef_row.wrapping_add(1);
            }
        } else {
            if writable == 0 {
                /* reader looking at undefined data */
                ERREXIT(cinfo, JERR_BAD_VIRTUAL_ACCESS);
            }
        }
    }
    /* Flag the buffer dirty if caller will write in it */
    if writable != 0 {
        (*ptr).dirty = 1;
    }
    /* Return address of proper part of the buffer */
    (*ptr).mem_buffer.wrapping_add((start_row as usize).wrapping_sub((*ptr).cur_start_row as usize))
}

unsafe fn access_virt_barray(
    cinfo: j_common_ptr,
    ptr: jvirt_barray_ptr,
    start_row: JDIMENSION,
    num_rows: JDIMENSION,
    writable: boolean,
) -> JBLOCKARRAY {
    /* Access the part of a virtual block array starting at start_row */
    /* and extending for num_rows rows.  writable is true if  */
    /* caller intends to modify the accessed area. */
    let end_row: JDIMENSION = start_row.wrapping_add(num_rows);
    let mut undef_row: JDIMENSION;

    /* debugging check */
    if end_row > (*ptr).rows_in_array
        || num_rows > (*ptr).maxaccess
        || (*ptr).mem_buffer.is_null()
    {
        ERREXIT(cinfo, JERR_BAD_VIRTUAL_ACCESS);
    }

    /* Make the desired part of the virtual array accessible */
    if start_row < (*ptr).cur_start_row || end_row > (*ptr).cur_start_row.wrapping_add((*ptr).rows_in_mem) {
        if (*ptr).b_s_open == 0 {
            ERREXIT(cinfo, JERR_VIRTUAL_BUG);
        }
        /* Flush old buffer contents if necessary */
        if (*ptr).dirty != 0 {
            do_barray_io(cinfo, ptr, 1);
            (*ptr).dirty = 0;
        }
        /* Decide what part of virtual array to access.
         * Algorithm: if target address > current window, assume forward scan,
         * load starting at target address.  If target address < current window,
         * assume backward scan, load so that target area is top of window.
         * Note that when switching from forward write to forward read, will have
         * start_row = 0, so the limiting case applies and we load from 0 anyway.
         */
        if start_row > (*ptr).cur_start_row {
            (*ptr).cur_start_row = start_row;
        } else {
            /* use long arithmetic here to avoid overflow & unsigned problems */
            let mut ltemp: c_long;

            ltemp = (end_row as c_long) - ((*ptr).rows_in_mem as c_long);
            if ltemp < 0 {
                ltemp = 0; /* don't fall off front end of file */
            }
            (*ptr).cur_start_row = ltemp as JDIMENSION;
        }
        /* Read in the selected part of the array.
         * During the initial write pass, we will do no actual read
         * because the selected part is all undefined.
         */
        do_barray_io(cinfo, ptr, 0);
    }
    /* Ensure the accessed part of the array is defined; prezero if needed.
     * To improve locality of access, we only prezero the part of the array
     * that the caller is about to access, not the entire in-memory array.
     */
    if (*ptr).first_undef_row < end_row {
        if (*ptr).first_undef_row < start_row {
            if writable != 0 {
                /* writer skipped over a section of array */
                ERREXIT(cinfo, JERR_BAD_VIRTUAL_ACCESS);
            }
            undef_row = start_row; /* but reader is allowed to read ahead */
        } else {
            undef_row = (*ptr).first_undef_row;
        }
        if writable != 0 {
            (*ptr).first_undef_row = end_row;
        }
        if (*ptr).pre_zero != 0 {
            let bytesperrow: usize = ((*ptr).blocksperrow as usize) * SIZEOF::<JBLOCK>();
            undef_row = undef_row.wrapping_sub((*ptr).cur_start_row); /* make indexes relative to buffer */
            let mut end_row_adjusted = end_row.wrapping_sub((*ptr).cur_start_row);
            while undef_row < end_row_adjusted {
                jzero_far(
                    *(*ptr).mem_buffer.wrapping_add(undef_row as usize) as *mut c_void,
                    bytesperrow,
                );
                undef_row = undef_row.wrapping_add(1);
            }
        } else {
            if writable == 0 {
                /* reader looking at undefined data */
                ERREXIT(cinfo, JERR_BAD_VIRTUAL_ACCESS);
            }
        }
    }
    /* Flag the buffer dirty if caller will write in it */
    if writable != 0 {
        (*ptr).dirty = 1;
    }
    /* Return address of proper part of the buffer */
    (*ptr).mem_buffer.wrapping_add((start_row as usize).wrapping_sub((*ptr).cur_start_row as usize))
}

extern "C" {
    fn jzero_far(target: *mut c_void, bytescount: usize);
}

/*
 * Release all objects belonging to a specified pool.
 */

unsafe fn free_pool(cinfo: j_common_ptr, pool_id: c_int) {
    let mem = (*cinfo).mem as my_mem_ptr;
    let mut shdr_ptr: small_pool_ptr;
    let mut lhdr_ptr: large_pool_ptr;
    let mut space_freed: usize;

    if pool_id < 0 || pool_id >= JPOOL_NUMPOOLS {
        ERREXIT1(cinfo, JERR_BAD_POOL_ID, pool_id); /* safety check */
    }

    #[cfg(feature = "MEM_STATS")]
    {
        if (*(*cinfo).err).trace_level > 1 {
            print_mem_stats(cinfo, pool_id); /* print pool's memory usage statistics */
        }
    }

    /* If freeing IMAGE pool, close any virtual arrays first */
    if pool_id == JPOOL_IMAGE {
        let mut sptr: jvirt_sarray_ptr;
        let mut bptr: jvirt_barray_ptr;

        sptr = (*mem).virt_sarray_list;
        while !sptr.is_null() {
            if (*sptr).b_s_open != 0 {
                /* there may be no backing store */
                (*sptr).b_s_open = 0; /* prevent recursive close if error */
                ((*(*sptr).b_s_info.close_backing_store)(cinfo, &mut (*sptr).b_s_info));
            }
            sptr = (*sptr).next;
        }
        (*mem).virt_sarray_list = core::ptr::null_mut();
        bptr = (*mem).virt_barray_list;
        while !bptr.is_null() {
            if (*bptr).b_s_open != 0 {
                /* there may be no backing store */
                (*bptr).b_s_open = 0; /* prevent recursive close if error */
                ((*(*bptr).b_s_info.close_backing_store)(cinfo, &mut (*bptr).b_s_info));
            }
            bptr = (*bptr).next;
        }
        (*mem).virt_barray_list = core::ptr::null_mut();
    }

    /* Release large objects */
    lhdr_ptr = (*mem).large_list[pool_id as usize];
    (*mem).large_list[pool_id as usize] = core::ptr::null_mut();

    while !lhdr_ptr.is_null() {
        let next_lhdr_ptr: large_pool_ptr = (*lhdr_ptr).hdr.next;
        space_freed = (*lhdr_ptr)
            .hdr
            .bytes_used
            .wrapping_add((*lhdr_ptr).hdr.bytes_left)
            .wrapping_add(SIZEOF::<large_pool_struct>());
        jpeg_free_large(cinfo, lhdr_ptr as *mut c_void, space_freed);
        (*mem).total_space_allocated = (*mem).total_space_allocated.wrapping_sub(space_freed as c_long);
        lhdr_ptr = next_lhdr_ptr;
    }

    /* Release small objects */
    shdr_ptr = (*mem).small_list[pool_id as usize];
    (*mem).small_list[pool_id as usize] = core::ptr::null_mut();

    while !shdr_ptr.is_null() {
        let next_shdr_ptr: small_pool_ptr = (*shdr_ptr).hdr.next;
        space_freed = (*shdr_ptr)
            .hdr
            .bytes_used
            .wrapping_add((*shdr_ptr).hdr.bytes_left)
            .wrapping_add(SIZEOF::<small_pool_struct>());
        jpeg_free_small(cinfo, shdr_ptr as *mut c_void, space_freed);
        (*mem).total_space_allocated = (*mem).total_space_allocated.wrapping_sub(space_freed as c_long);
        shdr_ptr = next_shdr_ptr;
    }
}

/*
 * Close up shop entirely.
 * Note that this cannot be called unless cinfo->mem is non-NULL.
 */

unsafe fn self_destruct(cinfo: j_common_ptr) {
    let mut pool: c_int;

    /* Close all backing store, release all memory.
     * Releasing pools in reverse order might help avoid fragmentation
     * with some (brain-damaged) malloc libraries.
     */
    pool = JPOOL_NUMPOOLS - 1;
    while pool >= JPOOL_PERMANENT {
        free_pool(cinfo, pool);
        pool -= 1;
    }

    /* Release the memory manager control block too. */
    jpeg_free_small(cinfo, (*cinfo).mem as *mut c_void, SIZEOF::<my_memory_mgr>());
    (*cinfo).mem = core::ptr::null_mut(); /* ensures I will be called only once */

    jpeg_mem_term(cinfo); /* system-dependent cleanup */
}

extern "C" {
    fn getenv(name: *const c_char) -> *const c_char;
    fn sscanf(s: *const c_char, format: *const c_char, ...) -> c_int;
}

/*
 * Memory manager initialization.
 * When this is called, only the error manager pointer is valid in cinfo!
 */

pub unsafe fn jinit_memory_mgr(cinfo: j_common_ptr) {
    let mut mem: my_mem_ptr;
    let mut max_to_use: c_long;
    let mut pool: c_int;
    let mut test_mac: usize;

    (*cinfo).mem = core::ptr::null_mut(); /* for safety if init fails */

    /* Check for configuration errors.
     * SIZEOF(ALIGN_TYPE) should be a power of 2; otherwise, it probably
     * doesn't reflect any real hardware alignment requirement.
     * The test is a little tricky: for X>0, X and X-1 have no one-bits
     * in common if and only if X is a power of 2, ie has only one one-bit.
     * Some compilers may give an "unreachable code" warning here; ignore it.
     */
    if (SIZEOF::<ALIGN_TYPE>() & (SIZEOF::<ALIGN_TYPE>().wrapping_sub(1))) != 0 {
        ERREXIT(cinfo, JERR_BAD_ALIGN_TYPE);
    }
    /* MAX_ALLOC_CHUNK must be representable as type size_t, and must be
     * a multiple of SIZEOF(ALIGN_TYPE).
     * Again, an "unreachable code" warning may be ignored here.
     * But a "constant too large" warning means you need to fix MAX_ALLOC_CHUNK.
     */
    test_mac = MAX_ALLOC_CHUNK as usize;
    if (test_mac as i64) != MAX_ALLOC_CHUNK || (MAX_ALLOC_CHUNK % (SIZEOF::<ALIGN_TYPE>() as i64)) != 0 {
        ERREXIT(cinfo, JERR_BAD_ALLOC_CHUNK);
    }

    max_to_use = jpeg_mem_init(cinfo); /* system-dependent initialization */

    /* Attempt to allocate memory manager's control block */
    mem = jpeg_get_small(cinfo, SIZEOF::<my_memory_mgr>()) as my_mem_ptr;

    if mem.is_null() {
        jpeg_mem_term(cinfo); /* system-dependent cleanup */
        ERREXIT1(cinfo, JERR_OUT_OF_MEMORY, 0);
    }

    /* OK, fill in the method pointers */
    (*mem).pub_.alloc_small = Some(alloc_small as unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void);
    (*mem).pub_.alloc_large = Some(alloc_large as unsafe extern "C" fn(j_common_ptr, c_int, usize) -> *mut c_void);
    (*mem).pub_.alloc_sarray = Some(alloc_sarray as unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> JSAMPARRAY);
    (*mem).pub_.alloc_barray = Some(alloc_barray as unsafe extern "C" fn(j_common_ptr, c_int, JDIMENSION, JDIMENSION) -> JBLOCKARRAY);
    (*mem).pub_.request_virt_sarray = Some(request_virt_sarray as unsafe extern "C" fn(j_common_ptr, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION) -> jvirt_sarray_ptr);
    (*mem).pub_.request_virt_barray = Some(request_virt_barray as unsafe extern "C" fn(j_common_ptr, c_int, boolean, JDIMENSION, JDIMENSION, JDIMENSION) -> jvirt_barray_ptr);
    (*mem).pub_.realize_virt_arrays = Some(realize_virt_arrays as unsafe extern "C" fn(j_common_ptr));
    (*mem).pub_.access_virt_sarray = Some(access_virt_sarray as unsafe extern "C" fn(j_common_ptr, jvirt_sarray_ptr, JDIMENSION, JDIMENSION, boolean) -> JSAMPARRAY);
    (*mem).pub_.access_virt_barray = Some(access_virt_barray as unsafe extern "C" fn(j_common_ptr, jvirt_barray_ptr, JDIMENSION, JDIMENSION, boolean) -> JBLOCKARRAY);
    (*mem).pub_.free_pool = Some(free_pool as unsafe extern "C" fn(j_common_ptr, c_int));
    (*mem).pub_.self_destruct = Some(self_destruct as unsafe extern "C" fn(j_common_ptr));

    /* Initialize working state */
    (*mem).pub_.max_memory_to_use = max_to_use;

    pool = JPOOL_NUMPOOLS - 1;
    while pool >= JPOOL_PERMANENT {
        (*mem).small_list[pool as usize] = core::ptr::null_mut();
        (*mem).large_list[pool as usize] = core::ptr::null_mut();
        pool -= 1;
    }
    (*mem).virt_sarray_list = core::ptr::null_mut();
    (*mem).virt_barray_list = core::ptr::null_mut();

    (*mem).total_space_allocated = SIZEOF::<my_memory_mgr>() as c_long;

    /* Declare ourselves open for business */
    (*cinfo).mem = &mut (*mem).pub_;

    /* Check for an environment variable JPEGMEM; if found, override the
     * default max_memory setting from jpeg_mem_init.  Note that the
     * surrounding application may again override this value.
     * If your system doesn't support getenv(), define NO_GETENV to disable
     * this feature.
     */
    #[cfg(not(feature = "NO_GETENV"))]
    {
        let memenv: *const c_char = getenv(b"JPEGMEM\0".as_ptr() as *const c_char);

        if !memenv.is_null() {
            let mut ch: c_char = b'x' as c_char;

            if sscanf(memenv, b"%ld%c\0".as_ptr() as *const c_char, &mut max_to_use, &mut ch) > 0 {
                if ch == b'm' as c_char || ch == b'M' as c_char {
                    max_to_use = max_to_use.wrapping_mul(1000);
                }
                (*mem).pub_.max_memory_to_use = max_to_use.wrapping_mul(1000);
            }
        }
    }
}
