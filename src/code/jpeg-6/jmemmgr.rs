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

// leave this as first line for PCH reasons...
//

use core::ffi::{c_int, c_void};
use core::mem::{size_of, align_of};
use core::ptr::{addr_of, addr_of_mut, null_mut};

// JPEG type stubs (normally from included headers jinclude.h, jpeglib.h, jmemsys.h)
// We define these as opaque/stub types since we're porting only this module

pub type JDIMENSION = c_int;
pub type JSAMPLE = u8;
pub type JBLOCK = [i16; 64];
pub type boolean = u8;

pub const FALSE: boolean = 0;
pub const TRUE: boolean = 1;

pub type JSAMPROW = *mut JSAMPLE;
pub type JSAMPARRAY = *mut JSAMPROW;

pub type JBLOCKROW = *mut JBLOCK;
pub type JBLOCKARRAY = *mut JBLOCKROW;

// Constants
const JPOOL_PERMANENT: c_int = 0;
const JPOOL_IMAGE: c_int = 1;
const JPOOL_NUMPOOLS: usize = 2;

const MAX_ALLOC_CHUNK: c_int = 0x7FFF_FFFF; // 2^31 - 1

// Alignment type (f64 like in C)
type ALIGN_TYPE = f64;

fn SIZEOF<T>() -> usize {
    size_of::<T>()
}

fn MIN(a: c_int, b: c_int) -> c_int {
    if a < b { a } else { b }
}

fn MIN_LONG(a: i64, b: i64) -> i64 {
    if a < b { a } else { b }
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

#[repr(C)]
pub union small_pool_struct {
    pub hdr: small_pool_hdr_fields,
    dummy: ALIGN_TYPE,
}

#[repr(C)]
pub struct small_pool_hdr_fields {
    pub next: *mut small_pool_struct,     // next in list of pools
    pub bytes_used: usize,                // how many bytes already used within pool
    pub bytes_left: usize,                // bytes still available in this pool
}

pub type small_pool_ptr = *mut small_pool_struct;

#[repr(C)]
pub union large_pool_struct {
    pub hdr: large_pool_hdr_fields,
    dummy: ALIGN_TYPE,
}

#[repr(C)]
pub struct large_pool_hdr_fields {
    pub next: *mut large_pool_struct,     // next in list of pools
    pub bytes_used: usize,                // how many bytes already used within pool
    pub bytes_left: usize,                // bytes still available in this pool
}

pub type large_pool_ptr = *mut large_pool_struct;


/*
 * Here is the full definition of a memory manager object.
 */

#[repr(C)]
pub struct my_memory_mgr {
    pub pub_: jpeg_memory_mgr,            // public fields

    // Each pool identifier (lifetime class) names a linked list of pools.
    pub small_list: [small_pool_ptr; JPOOL_NUMPOOLS],
    pub large_list: [large_pool_ptr; JPOOL_NUMPOOLS],

    // Since we only have one lifetime class of virtual arrays, only one
    // linked list is necessary (for each datatype).  Note that the virtual
    // array control blocks being linked together are actually stored somewhere
    // in the small-pool list.
    pub virt_sarray_list: jvirt_sarray_ptr,
    pub virt_barray_list: jvirt_barray_ptr,

    // This counts total space obtained from jpeg_get_small/large
    pub total_space_allocated: i64,

    // alloc_sarray and alloc_barray set this value for use by virtual
    // array routines.
    pub last_rowsperchunk: JDIMENSION,    // from most recent alloc_sarray/barray
}

pub type my_mem_ptr = *mut my_memory_mgr;

#[repr(C)]
pub struct jpeg_memory_mgr {
    pub alloc_small: *const c_void,
    pub alloc_large: *const c_void,
    pub alloc_sarray: *const c_void,
    pub alloc_barray: *const c_void,
    pub request_virt_sarray: *const c_void,
    pub request_virt_barray: *const c_void,
    pub realize_virt_arrays: *const c_void,
    pub access_virt_sarray: *const c_void,
    pub access_virt_barray: *const c_void,
    pub free_pool: *const c_void,
    pub self_destruct: *const c_void,
    pub max_memory_to_use: i64,
}


/*
 * The control blocks for virtual arrays.
 * Note that these blocks are allocated in the "small" pool area.
 * System-dependent info for the associated backing store (if any) is hidden
 * inside the backing_store_info struct.
 */

#[repr(C)]
pub struct jvirt_sarray_control {
    pub mem_buffer: JSAMPARRAY,           // => the in-memory buffer
    pub rows_in_array: JDIMENSION,        // total virtual array height
    pub samplesperrow: JDIMENSION,        // width of array (and of memory buffer)
    pub maxaccess: JDIMENSION,            // max rows accessed by access_virt_sarray
    pub rows_in_mem: JDIMENSION,          // height of memory buffer
    pub rowsperchunk: JDIMENSION,         // allocation chunk size in mem_buffer
    pub cur_start_row: JDIMENSION,        // first logical row # in the buffer
    pub first_undef_row: JDIMENSION,      // row # of first uninitialized row
    pub pre_zero: boolean,                // pre-zero mode requested?
    pub dirty: boolean,                   // do current buffer contents need written?
    pub b_s_open: boolean,                // is backing-store data valid?
    pub next: jvirt_sarray_ptr,           // link to next virtual sarray control block
    pub b_s_info: backing_store_info,     // System-dependent control info
}

pub type jvirt_sarray_ptr = *mut jvirt_sarray_control;

#[repr(C)]
pub struct jvirt_barray_control {
    pub mem_buffer: JBLOCKARRAY,          // => the in-memory buffer
    pub rows_in_array: JDIMENSION,        // total virtual array height
    pub blocksperrow: JDIMENSION,         // width of array (and of memory buffer)
    pub maxaccess: JDIMENSION,            // max rows accessed by access_virt_barray
    pub rows_in_mem: JDIMENSION,          // height of memory buffer
    pub rowsperchunk: JDIMENSION,         // allocation chunk size in mem_buffer
    pub cur_start_row: JDIMENSION,        // first logical row # in the buffer
    pub first_undef_row: JDIMENSION,      // row # of first uninitialized row
    pub pre_zero: boolean,                // pre-zero mode requested?
    pub dirty: boolean,                   // do current buffer contents need written?
    pub b_s_open: boolean,                // is backing-store data valid?
    pub next: jvirt_barray_ptr,           // link to next virtual barray control block
    pub b_s_info: backing_store_info,     // System-dependent control info
}

pub type jvirt_barray_ptr = *mut jvirt_barray_control;

#[repr(C)]
pub struct backing_store_info {
    pub read_backing_store: *const c_void,
    pub write_backing_store: *const c_void,
    pub close_backing_store: *const c_void,
}


// External JPEG functions (stubs for ABI compatibility)
extern "C" {
    fn jpeg_get_small(cinfo: *const c_void, sizeofobject: usize) -> *mut c_void;
    fn jpeg_get_large(cinfo: *const c_void, sizeofobject: usize) -> *mut c_void;
    fn jpeg_free_small(cinfo: *const c_void, object: *const c_void, size: usize);
    fn jpeg_free_large(cinfo: *const c_void, object: *const c_void, size: usize);
    fn jpeg_mem_available(cinfo: *const c_void, min_bytes_needed: i64, max_bytes_needed: i64,
                          already_allocated: i64) -> i64;
    fn jpeg_mem_init(cinfo: *const c_void) -> i64;
    fn jpeg_mem_term(cinfo: *const c_void);
    fn jpeg_open_backing_store(cinfo: *const c_void, info: *mut backing_store_info,
                               total_bytes_needed: i64);
}

// Macros/stubs
// Error handling stubs (normally from jinclude.h)
macro_rules! ERREXIT {
    ($cinfo:expr, $err:expr) => {
        // Stub: in real code this would call error handler
    }
}

macro_rules! ERREXIT1 {
    ($cinfo:expr, $err:expr, $p1:expr) => {
        // Stub: in real code this would call error handler
    }
}

const JERR_OUT_OF_MEMORY: i32 = 1;
const JERR_BAD_POOL_ID: i32 = 2;
const JERR_WIDTH_OVERFLOW: i32 = 3;
const JERR_BAD_VIRTUAL_ACCESS: i32 = 4;
const JERR_VIRTUAL_BUG: i32 = 5;
const JERR_BAD_ALIGN_TYPE: i32 = 6;
const JERR_BAD_ALLOC_CHUNK: i32 = 7;


#[cfg(feature = "mem_stats")]
fn print_mem_stats(cinfo: *const c_void, pool_id: c_int) {
    unsafe {
        let mem = cinfo as *const my_memory_mgr;
        let mut shdr_ptr: small_pool_ptr;
        let mut lhdr_ptr: large_pool_ptr;

        /* Since this is only a debugging stub, we can cheat a little by using
         * fprintf directly rather than going through the trace message code.
         * This is helpful because message parm array can't handle longs.
         */
        eprintln!("Freeing pool {}, total space = {}\n",
                  pool_id, (*mem).total_space_allocated);

        lhdr_ptr = (*mem).large_list[pool_id as usize];
        while !lhdr_ptr.is_null() {
            eprintln!("  Large chunk used {}\n",
                    (*lhdr_ptr).hdr.bytes_used);
            lhdr_ptr = (*lhdr_ptr).hdr.next;
        }

        shdr_ptr = (*mem).small_list[pool_id as usize];
        while !shdr_ptr.is_null() {
            eprintln!("  Small chunk used {} free {}\n",
                    (*shdr_ptr).hdr.bytes_used,
                    (*shdr_ptr).hdr.bytes_left);
            shdr_ptr = (*shdr_ptr).hdr.next;
        }
    }
}


fn out_of_memory(cinfo: *const c_void, which: c_int) {
    /* Report an out-of-memory error and stop execution */
    /* If we compiled MEM_STATS support, report alloc requests before dying */
    #[cfg(feature = "mem_stats")]
    {
        unsafe {
            let cinfo_info = cinfo as *mut j_common_ptr;
            (*(*cinfo_info).err).trace_level = 2;	/* force self_destruct to report stats */
        }
    }
    ERREXIT1!(cinfo, JERR_OUT_OF_MEMORY, which);
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

static FIRST_POOL_SLOP: [usize; JPOOL_NUMPOOLS] = [
    1600,           // first PERMANENT pool
    16000           // first IMAGE pool
];

static EXTRA_POOL_SLOP: [usize; JPOOL_NUMPOOLS] = [
    0,              // additional PERMANENT pools
    5000            // additional IMAGE pools
];

const MIN_SLOP: usize = 50;     // greater than 0 to avoid futile looping


fn alloc_small(cinfo: *const c_void, pool_id: c_int, sizeofobject: usize) -> *mut c_void {
    /* Allocate a "small" object */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let mut hdr_ptr: small_pool_ptr;
        let mut prev_hdr_ptr: small_pool_ptr;
        let mut data_ptr: *mut u8;
        let mut odd_bytes: usize;
        let mut min_request: usize;
        let mut slop: usize;
        let mut sizeofobject = sizeofobject;

        /* Check for unsatisfiable request (do now to ensure no overflow below) */
        if sizeofobject > (MAX_ALLOC_CHUNK as usize).wrapping_sub(SIZEOF::<small_pool_struct>()) {
            out_of_memory(cinfo, 1);	/* request exceeds malloc's ability */
        }

        /* Round up the requested size to a multiple of SIZEOF(ALIGN_TYPE) */
        odd_bytes = sizeofobject % SIZEOF::<ALIGN_TYPE>();
        if odd_bytes > 0 {
            sizeofobject += SIZEOF::<ALIGN_TYPE>() - odd_bytes;
        }

        /* See if space is available in any existing pool */
        if pool_id < 0 || pool_id as usize >= JPOOL_NUMPOOLS {
            ERREXIT1!(cinfo, JERR_BAD_POOL_ID, pool_id);	/* safety check */
        }
        prev_hdr_ptr = null_mut();
        hdr_ptr = (*mem).small_list[pool_id as usize];
        while !hdr_ptr.is_null() {
            if (*hdr_ptr).hdr.bytes_left >= sizeofobject {
                break;			/* found pool with enough space */
            }
            prev_hdr_ptr = hdr_ptr;
            hdr_ptr = (*hdr_ptr).hdr.next;
        }

        /* Time to make a new pool? */
        if hdr_ptr.is_null() {
            /* min_request is what we need now, slop is what will be leftover */
            min_request = sizeofobject + SIZEOF::<small_pool_struct>();
            if prev_hdr_ptr.is_null() {	/* first pool in class? */
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
                hdr_ptr = jpeg_get_small(cinfo, min_request + slop) as small_pool_ptr;
                if !hdr_ptr.is_null() {
                    break;
                }
                slop /= 2;
                if slop < MIN_SLOP {	/* give up when it gets real small */
                    out_of_memory(cinfo, 2); /* jpeg_get_small failed */
                }
            }
            (*mem).total_space_allocated += (min_request + slop) as i64;
            /* Success, initialize the new pool header and add to end of list */
            (*hdr_ptr).hdr.next = null_mut();
            (*hdr_ptr).hdr.bytes_used = 0;
            (*hdr_ptr).hdr.bytes_left = sizeofobject + slop;
            if prev_hdr_ptr.is_null() {	/* first pool in class? */
                (*mem).small_list[pool_id as usize] = hdr_ptr;
            } else {
                (*prev_hdr_ptr).hdr.next = hdr_ptr;
            }
        }

        /* OK, allocate the object from the current pool */
        data_ptr = (hdr_ptr as *mut u8).offset(SIZEOF::<small_pool_struct>() as isize); /* point to first data byte in pool */
        data_ptr = data_ptr.offset((*hdr_ptr).hdr.bytes_used as isize); /* point to place for object */
        (*hdr_ptr).hdr.bytes_used += sizeofobject;
        (*hdr_ptr).hdr.bytes_left -= sizeofobject;

        data_ptr as *mut c_void
    }
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

fn alloc_large(cinfo: *const c_void, pool_id: c_int, sizeofobject: usize) -> *mut c_void {
    /* Allocate a "large" object */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let mut hdr_ptr: large_pool_ptr;
        let mut odd_bytes: usize;
        let mut sizeofobject = sizeofobject;

        /* Check for unsatisfiable request (do now to ensure no overflow below) */
        if sizeofobject > (MAX_ALLOC_CHUNK as usize).wrapping_sub(SIZEOF::<large_pool_struct>()) {
            out_of_memory(cinfo, 3);	/* request exceeds malloc's ability */
        }

        /* Round up the requested size to a multiple of SIZEOF(ALIGN_TYPE) */
        odd_bytes = sizeofobject % SIZEOF::<ALIGN_TYPE>();
        if odd_bytes > 0 {
            sizeofobject += SIZEOF::<ALIGN_TYPE>() - odd_bytes;
        }

        /* Always make a new pool */
        if pool_id < 0 || pool_id as usize >= JPOOL_NUMPOOLS {
            ERREXIT1!(cinfo, JERR_BAD_POOL_ID, pool_id);	/* safety check */
        }

        hdr_ptr = jpeg_get_large(cinfo, sizeofobject + SIZEOF::<large_pool_struct>()) as large_pool_ptr;
        if hdr_ptr.is_null() {
            out_of_memory(cinfo, 4);	/* jpeg_get_large failed */
        }
        (*mem).total_space_allocated += (sizeofobject + SIZEOF::<large_pool_struct>()) as i64;

        /* Success, initialize the new pool header and add to list */
        (*hdr_ptr).hdr.next = (*mem).large_list[pool_id as usize];
        /* We maintain space counts in each pool header for statistical purposes,
         * even though they are not needed for allocation.
         */
        (*hdr_ptr).hdr.bytes_used = sizeofobject;
        (*hdr_ptr).hdr.bytes_left = 0;
        (*mem).large_list[pool_id as usize] = hdr_ptr;

        (hdr_ptr as *mut u8).offset(SIZEOF::<large_pool_struct>() as isize) as *mut c_void /* point to first data byte in pool */
    }
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

fn alloc_sarray(cinfo: *const c_void, pool_id: c_int,
                samplesperrow: JDIMENSION, numrows: JDIMENSION) -> JSAMPARRAY {
    /* Allocate a 2-D sample array */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let result: JSAMPARRAY;
        let mut workspace: JSAMPROW;
        let mut rowsperchunk: JDIMENSION;
        let mut currow: JDIMENSION;
        let mut i: JDIMENSION;
        let ltemp: i64;

        /* Calculate max # of rows allowed in one allocation chunk */
        ltemp = ((MAX_ALLOC_CHUNK as i64).wrapping_sub(SIZEOF::<large_pool_struct>() as i64)) /
                ((samplesperrow as i64).wrapping_mul(SIZEOF::<JSAMPLE>() as i64));
        if ltemp <= 0 {
            ERREXIT!(cinfo, JERR_WIDTH_OVERFLOW);
        }
        if ltemp < (numrows as i64) {
            rowsperchunk = ltemp as JDIMENSION;
        } else {
            rowsperchunk = numrows;
        }
        (*mem).last_rowsperchunk = rowsperchunk;

        /* Get space for row pointers (small object) */
        result = alloc_small(cinfo, pool_id,
                            (numrows as usize).wrapping_mul(SIZEOF::<JSAMPROW>())) as JSAMPARRAY;

        /* Get the rows themselves (large objects) */
        currow = 0;
        while currow < numrows {
            rowsperchunk = MIN(rowsperchunk, numrows - currow);
            workspace = alloc_large(cinfo, pool_id,
                ((rowsperchunk as usize).wrapping_mul(samplesperrow as usize))
                  .wrapping_mul(SIZEOF::<JSAMPLE>())) as JSAMPROW;
            i = rowsperchunk;
            while i > 0 {
                *result.offset(currow as isize) = workspace;
                workspace = workspace.offset(samplesperrow as isize);
                currow += 1;
                i -= 1;
            }
        }

        result
    }
}


/*
 * Creation of 2-D coefficient-block arrays.
 * This is essentially the same as the code for sample arrays, above.
 */

fn alloc_barray(cinfo: *const c_void, pool_id: c_int,
                blocksperrow: JDIMENSION, numrows: JDIMENSION) -> JBLOCKARRAY {
    /* Allocate a 2-D coefficient-block array */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let result: JBLOCKARRAY;
        let mut workspace: JBLOCKROW;
        let mut rowsperchunk: JDIMENSION;
        let mut currow: JDIMENSION;
        let mut i: JDIMENSION;
        let ltemp: i64;

        /* Calculate max # of rows allowed in one allocation chunk */
        ltemp = ((MAX_ALLOC_CHUNK as i64).wrapping_sub(SIZEOF::<large_pool_struct>() as i64)) /
                ((blocksperrow as i64).wrapping_mul(SIZEOF::<JBLOCK>() as i64));
        if ltemp <= 0 {
            ERREXIT!(cinfo, JERR_WIDTH_OVERFLOW);
        }
        if ltemp < (numrows as i64) {
            rowsperchunk = ltemp as JDIMENSION;
        } else {
            rowsperchunk = numrows;
        }
        (*mem).last_rowsperchunk = rowsperchunk;

        /* Get space for row pointers (small object) */
        result = alloc_small(cinfo, pool_id,
                            (numrows as usize).wrapping_mul(SIZEOF::<JBLOCKROW>())) as JBLOCKARRAY;

        /* Get the rows themselves (large objects) */
        currow = 0;
        while currow < numrows {
            rowsperchunk = MIN(rowsperchunk, numrows - currow);
            workspace = alloc_large(cinfo, pool_id,
                ((rowsperchunk as usize).wrapping_mul(blocksperrow as usize))
                  .wrapping_mul(SIZEOF::<JBLOCK>())) as JBLOCKROW;
            i = rowsperchunk;
            while i > 0 {
                *result.offset(currow as isize) = workspace;
                workspace = workspace.offset(blocksperrow as isize);
                currow += 1;
                i -= 1;
            }
        }

        result
    }
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


fn request_virt_sarray(cinfo: *const c_void, pool_id: c_int, pre_zero: boolean,
                       samplesperrow: JDIMENSION, numrows: JDIMENSION,
                       maxaccess: JDIMENSION) -> jvirt_sarray_ptr {
    /* Request a virtual 2-D sample array */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let result: jvirt_sarray_ptr;

        /* Only IMAGE-lifetime virtual arrays are currently supported */
        if pool_id != JPOOL_IMAGE {
            ERREXIT1!(cinfo, JERR_BAD_POOL_ID, pool_id);	/* safety check */
        }

        /* get control block */
        result = alloc_small(cinfo, pool_id,
                            SIZEOF::<jvirt_sarray_control>()) as jvirt_sarray_ptr;

        (*result).mem_buffer = null_mut();	/* marks array not yet realized */
        (*result).rows_in_array = numrows;
        (*result).samplesperrow = samplesperrow;
        (*result).maxaccess = maxaccess;
        (*result).pre_zero = pre_zero;
        (*result).b_s_open = FALSE;	/* no associated backing-store object */
        (*result).next = (*mem).virt_sarray_list; /* add to list of virtual arrays */
        (*mem).virt_sarray_list = result;

        result
    }
}


fn request_virt_barray(cinfo: *const c_void, pool_id: c_int, pre_zero: boolean,
                       blocksperrow: JDIMENSION, numrows: JDIMENSION,
                       maxaccess: JDIMENSION) -> jvirt_barray_ptr {
    /* Request a virtual 2-D coefficient-block array */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let result: jvirt_barray_ptr;

        /* Only IMAGE-lifetime virtual arrays are currently supported */
        if pool_id != JPOOL_IMAGE {
            ERREXIT1!(cinfo, JERR_BAD_POOL_ID, pool_id);	/* safety check */
        }

        /* get control block */
        result = alloc_small(cinfo, pool_id,
                            SIZEOF::<jvirt_barray_control>()) as jvirt_barray_ptr;

        (*result).mem_buffer = null_mut();	/* marks array not yet realized */
        (*result).rows_in_array = numrows;
        (*result).blocksperrow = blocksperrow;
        (*result).maxaccess = maxaccess;
        (*result).pre_zero = pre_zero;
        (*result).b_s_open = FALSE;	/* no associated backing-store object */
        (*result).next = (*mem).virt_barray_list; /* add to list of virtual arrays */
        (*mem).virt_barray_list = result;

        result
    }
}


fn realize_virt_arrays(cinfo: *const c_void) {
    /* Allocate the in-memory buffers for any unrealized virtual arrays */
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let mut space_per_minheight: i64;
        let mut maximum_space: i64;
        let avail_mem: i64;
        let mut max_minheights: i64;
        let mut minheights: i64;
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
            if (*sptr).mem_buffer.is_null() { /* if not realized yet */
                space_per_minheight += ((*sptr).maxaccess as i64).wrapping_mul(
                                        ((*sptr).samplesperrow as i64).wrapping_mul(SIZEOF::<JSAMPLE>() as i64));
                maximum_space += ((*sptr).rows_in_array as i64).wrapping_mul(
                                ((*sptr).samplesperrow as i64).wrapping_mul(SIZEOF::<JSAMPLE>() as i64));
            }
            sptr = (*sptr).next;
        }
        bptr = (*mem).virt_barray_list;
        while !bptr.is_null() {
            if (*bptr).mem_buffer.is_null() { /* if not realized yet */
                space_per_minheight += ((*bptr).maxaccess as i64).wrapping_mul(
                                        ((*bptr).blocksperrow as i64).wrapping_mul(SIZEOF::<JBLOCK>() as i64));
                maximum_space += ((*bptr).rows_in_array as i64).wrapping_mul(
                                ((*bptr).blocksperrow as i64).wrapping_mul(SIZEOF::<JBLOCK>() as i64));
            }
            bptr = (*bptr).next;
        }

        if space_per_minheight <= 0 {
            return;			/* no unrealized arrays, no work */
        }

        /* Determine amount of memory to actually use; this is system-dependent. */
        let avail_mem = jpeg_mem_available(cinfo, space_per_minheight, maximum_space,
                                           (*mem).total_space_allocated);

        /* If the maximum space needed is available, make all the buffers full
         * height; otherwise parcel it out with the same number of minheights
         * in each buffer.
         */
        if avail_mem >= maximum_space {
            max_minheights = 1000000000i64;
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
            if (*sptr).mem_buffer.is_null() { /* if not realized yet */
                minheights = ((((*sptr).rows_in_array as i64).wrapping_sub(1i64)) / ((*sptr).maxaccess as i64)).wrapping_add(1i64);
                if minheights <= max_minheights {
                    /* This buffer fits in memory */
                    (*sptr).rows_in_mem = (*sptr).rows_in_array;
                } else {
                    /* It doesn't fit in memory, create backing store. */
                    (*sptr).rows_in_mem = (max_minheights.wrapping_mul((*sptr).maxaccess as i64)) as JDIMENSION;
                    jpeg_open_backing_store(cinfo, &mut (*sptr).b_s_info,
                                           ((*sptr).rows_in_array as i64).wrapping_mul(
                                            ((*sptr).samplesperrow as i64).wrapping_mul(
                                             SIZEOF::<JSAMPLE>() as i64)));
                    (*sptr).b_s_open = TRUE;
                }
                (*sptr).mem_buffer = alloc_sarray(cinfo, JPOOL_IMAGE,
                                                  (*sptr).samplesperrow, (*sptr).rows_in_mem);
                (*sptr).rowsperchunk = (*mem).last_rowsperchunk;
                (*sptr).cur_start_row = 0;
                (*sptr).first_undef_row = 0;
                (*sptr).dirty = FALSE;
            }
            sptr = (*sptr).next;
        }

        bptr = (*mem).virt_barray_list;
        while !bptr.is_null() {
            if (*bptr).mem_buffer.is_null() { /* if not realized yet */
                minheights = ((((*bptr).rows_in_array as i64).wrapping_sub(1i64)) / ((*bptr).maxaccess as i64)).wrapping_add(1i64);
                if minheights <= max_minheights {
                    /* This buffer fits in memory */
                    (*bptr).rows_in_mem = (*bptr).rows_in_array;
                } else {
                    /* It doesn't fit in memory, create backing store. */
                    (*bptr).rows_in_mem = (max_minheights.wrapping_mul((*bptr).maxaccess as i64)) as JDIMENSION;
                    jpeg_open_backing_store(cinfo, &mut (*bptr).b_s_info,
                                           ((*bptr).rows_in_array as i64).wrapping_mul(
                                            ((*bptr).blocksperrow as i64).wrapping_mul(
                                             SIZEOF::<JBLOCK>() as i64)));
                    (*bptr).b_s_open = TRUE;
                }
                (*bptr).mem_buffer = alloc_barray(cinfo, JPOOL_IMAGE,
                                                  (*bptr).blocksperrow, (*bptr).rows_in_mem);
                (*bptr).rowsperchunk = (*mem).last_rowsperchunk;
                (*bptr).cur_start_row = 0;
                (*bptr).first_undef_row = 0;
                (*bptr).dirty = FALSE;
            }
            bptr = (*bptr).next;
        }
    }
}


fn do_sarray_io(cinfo: *const c_void, ptr: jvirt_sarray_ptr, writing: boolean) {
    /* Do backing store read or write of a virtual sample array */
    unsafe {
        let mut bytesperrow: i64;
        let mut file_offset: i64;
        let mut byte_count: i64;
        let mut rows: i64;
        let mut thisrow: i64;
        let mut i: i64;

        bytesperrow = ((*ptr).samplesperrow as i64).wrapping_mul(SIZEOF::<JSAMPLE>() as i64);
        file_offset = ((*ptr).cur_start_row as i64).wrapping_mul(bytesperrow);
        /* Loop to read or write each allocation chunk in mem_buffer */
        i = 0;
        while i < ((*ptr).rows_in_mem as i64) {
            /* One chunk, but check for short chunk at end of buffer */
            rows = MIN_LONG(((*ptr).rowsperchunk as i64), ((*ptr).rows_in_mem as i64).wrapping_sub(i));
            /* Transfer no more than is currently defined */
            thisrow = ((*ptr).cur_start_row as i64).wrapping_add(i);
            rows = MIN_LONG(rows, ((*ptr).first_undef_row as i64).wrapping_sub(thisrow));
            /* Transfer no more than fits in file */
            rows = MIN_LONG(rows, ((*ptr).rows_in_array as i64).wrapping_sub(thisrow));
            if rows <= 0 {		/* this chunk might be past end of file! */
                break;
            }
            byte_count = rows.wrapping_mul(bytesperrow);
            if writing != 0 {
                // Call write_backing_store via function pointer
                // (*ptr->b_s_info.write_backing_store)(cinfo, &ptr->b_s_info,
                //                                       (void FAR *) ptr->mem_buffer[i],
                //                                       file_offset, byte_count);
            } else {
                // Call read_backing_store via function pointer
                // (*ptr->b_s_info.read_backing_store)(cinfo, &ptr->b_s_info,
                //                                      (void FAR *) ptr->mem_buffer[i],
                //                                      file_offset, byte_count);
            }
            file_offset += byte_count;
            i += (*ptr).rowsperchunk as i64;
        }
    }
}


fn do_barray_io(cinfo: *const c_void, ptr: jvirt_barray_ptr, writing: boolean) {
    /* Do backing store read or write of a virtual coefficient-block array */
    unsafe {
        let mut bytesperrow: i64;
        let mut file_offset: i64;
        let mut byte_count: i64;
        let mut rows: i64;
        let mut thisrow: i64;
        let mut i: i64;

        bytesperrow = ((*ptr).blocksperrow as i64).wrapping_mul(SIZEOF::<JBLOCK>() as i64);
        file_offset = ((*ptr).cur_start_row as i64).wrapping_mul(bytesperrow);
        /* Loop to read or write each allocation chunk in mem_buffer */
        i = 0;
        while i < ((*ptr).rows_in_mem as i64) {
            /* One chunk, but check for short chunk at end of buffer */
            rows = MIN_LONG(((*ptr).rowsperchunk as i64), ((*ptr).rows_in_mem as i64).wrapping_sub(i));
            /* Transfer no more than is currently defined */
            thisrow = ((*ptr).cur_start_row as i64).wrapping_add(i);
            rows = MIN_LONG(rows, ((*ptr).first_undef_row as i64).wrapping_sub(thisrow));
            /* Transfer no more than fits in file */
            rows = MIN_LONG(rows, ((*ptr).rows_in_array as i64).wrapping_sub(thisrow));
            if rows <= 0 {		/* this chunk might be past end of file! */
                break;
            }
            byte_count = rows.wrapping_mul(bytesperrow);
            if writing != 0 {
                // Call write_backing_store via function pointer
                // (*ptr->b_s_info.write_backing_store)(cinfo, &ptr->b_s_info,
                //                                       (void FAR *) ptr->mem_buffer[i],
                //                                       file_offset, byte_count);
            } else {
                // Call read_backing_store via function pointer
                // (*ptr->b_s_info.read_backing_store)(cinfo, &ptr->b_s_info,
                //                                      (void FAR *) ptr->mem_buffer[i],
                //                                      file_offset, byte_count);
            }
            file_offset += byte_count;
            i += (*ptr).rowsperchunk as i64;
        }
    }
}


fn access_virt_sarray(cinfo: *const c_void, ptr: jvirt_sarray_ptr,
                      start_row: JDIMENSION, num_rows: JDIMENSION,
                      writable: boolean) -> JSAMPARRAY {
    /* Access the part of a virtual sample array starting at start_row */
    /* and extending for num_rows rows.  writable is true if  */
    /* caller intends to modify the accessed area. */
    unsafe {
        let mut end_row = start_row + num_rows;
        let mut undef_row: JDIMENSION;

        /* debugging check */
        if end_row > (*ptr).rows_in_array || num_rows > (*ptr).maxaccess ||
            (*ptr).mem_buffer.is_null() {
            ERREXIT!(cinfo, JERR_BAD_VIRTUAL_ACCESS);
        }

        /* Make the desired part of the virtual array accessible */
        if start_row < (*ptr).cur_start_row ||
            end_row > (*ptr).cur_start_row + (*ptr).rows_in_mem {
            if (*ptr).b_s_open == FALSE {
                ERREXIT!(cinfo, JERR_VIRTUAL_BUG);
            }
            /* Flush old buffer contents if necessary */
            if (*ptr).dirty != FALSE {
                do_sarray_io(cinfo, ptr, TRUE);
                (*ptr).dirty = FALSE;
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
                let mut ltemp: i64;

                ltemp = (end_row as i64).wrapping_sub((*ptr).rows_in_mem as i64);
                if ltemp < 0 {
                    ltemp = 0;		/* don't fall off front end of file */
                }
                (*ptr).cur_start_row = ltemp as JDIMENSION;
            }
            /* Read in the selected part of the array.
             * During the initial write pass, we will do no actual read
             * because the selected part is all undefined.
             */
            do_sarray_io(cinfo, ptr, FALSE);
        }
        /* Ensure the accessed part of the array is defined; prezero if needed.
         * To improve locality of access, we only prezero the part of the array
         * that the caller is about to access, not the entire in-memory array.
         */
        if (*ptr).first_undef_row < end_row {
            if (*ptr).first_undef_row < start_row {
                if writable != FALSE {		/* writer skipped over a section of array */
                    ERREXIT!(cinfo, JERR_BAD_VIRTUAL_ACCESS);
                }
                undef_row = start_row;	/* but reader is allowed to read ahead */
            } else {
                undef_row = (*ptr).first_undef_row;
            }
            if writable != FALSE {
                (*ptr).first_undef_row = end_row;
            }
            if (*ptr).pre_zero != FALSE {
                let bytesperrow: usize = ((*ptr).samplesperrow as usize).wrapping_mul(SIZEOF::<JSAMPLE>());
                undef_row -= (*ptr).cur_start_row; /* make indexes relative to buffer */
                end_row -= (*ptr).cur_start_row;
                while undef_row < end_row {
                    // jzero_far((void FAR *) ptr->mem_buffer[undef_row], bytesperrow);
                    undef_row += 1;
                }
            } else {
                if writable == FALSE {		/* reader looking at undefined data */
                    ERREXIT!(cinfo, JERR_BAD_VIRTUAL_ACCESS);
                }
            }
        }
        /* Flag the buffer dirty if caller will write in it */
        if writable != FALSE {
            (*ptr).dirty = TRUE;
        }
        /* Return address of proper part of the buffer */
        (*ptr).mem_buffer.offset((start_row - (*ptr).cur_start_row) as isize)
    }
}


fn access_virt_barray(cinfo: *const c_void, ptr: jvirt_barray_ptr,
                      start_row: JDIMENSION, num_rows: JDIMENSION,
                      writable: boolean) -> JBLOCKARRAY {
    /* Access the part of a virtual block array starting at start_row */
    /* and extending for num_rows rows.  writable is true if  */
    /* caller intends to modify the accessed area. */
    unsafe {
        let mut end_row = start_row + num_rows;
        let mut undef_row: JDIMENSION;

        /* debugging check */
        if end_row > (*ptr).rows_in_array || num_rows > (*ptr).maxaccess ||
            (*ptr).mem_buffer.is_null() {
            ERREXIT!(cinfo, JERR_BAD_VIRTUAL_ACCESS);
        }

        /* Make the desired part of the virtual array accessible */
        if start_row < (*ptr).cur_start_row ||
            end_row > (*ptr).cur_start_row + (*ptr).rows_in_mem {
            if (*ptr).b_s_open == FALSE {
                ERREXIT!(cinfo, JERR_VIRTUAL_BUG);
            }
            /* Flush old buffer contents if necessary */
            if (*ptr).dirty != FALSE {
                do_barray_io(cinfo, ptr, TRUE);
                (*ptr).dirty = FALSE;
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
                let mut ltemp: i64;

                ltemp = (end_row as i64).wrapping_sub((*ptr).rows_in_mem as i64);
                if ltemp < 0 {
                    ltemp = 0;		/* don't fall off front end of file */
                }
                (*ptr).cur_start_row = ltemp as JDIMENSION;
            }
            /* Read in the selected part of the array.
             * During the initial write pass, we will do no actual read
             * because the selected part is all undefined.
             */
            do_barray_io(cinfo, ptr, FALSE);
        }
        /* Ensure the accessed part of the array is defined; prezero if needed.
         * To improve locality of access, we only prezero the part of the array
         * that the caller is about to access, not the entire in-memory array.
         */
        if (*ptr).first_undef_row < end_row {
            if (*ptr).first_undef_row < start_row {
                if writable != FALSE {		/* writer skipped over a section of array */
                    ERREXIT!(cinfo, JERR_BAD_VIRTUAL_ACCESS);
                }
                undef_row = start_row;	/* but reader is allowed to read ahead */
            } else {
                undef_row = (*ptr).first_undef_row;
            }
            if writable != FALSE {
                (*ptr).first_undef_row = end_row;
            }
            if (*ptr).pre_zero != FALSE {
                let bytesperrow: usize = ((*ptr).blocksperrow as usize).wrapping_mul(SIZEOF::<JBLOCK>());
                undef_row -= (*ptr).cur_start_row; /* make indexes relative to buffer */
                end_row -= (*ptr).cur_start_row;
                while undef_row < end_row {
                    // jzero_far((void FAR *) ptr->mem_buffer[undef_row], bytesperrow);
                    undef_row += 1;
                }
            } else {
                if writable == FALSE {		/* reader looking at undefined data */
                    ERREXIT!(cinfo, JERR_BAD_VIRTUAL_ACCESS);
                }
            }
        }
        /* Flag the buffer dirty if caller will write in it */
        if writable != FALSE {
            (*ptr).dirty = TRUE;
        }
        /* Return address of proper part of the buffer */
        (*ptr).mem_buffer.offset((start_row - (*ptr).cur_start_row) as isize)
    }
}


/*
 * Release all objects belonging to a specified pool.
 */

fn free_pool(cinfo: *const c_void, pool_id: c_int) {
    unsafe {
        let mem = cinfo as *mut my_memory_mgr;
        let mut shdr_ptr: small_pool_ptr;
        let mut lhdr_ptr: large_pool_ptr;
        let mut space_freed: usize;

        if pool_id < 0 || pool_id as usize >= JPOOL_NUMPOOLS {
            ERREXIT1!(cinfo, JERR_BAD_POOL_ID, pool_id);	/* safety check */
        }

        #[cfg(feature = "mem_stats")]
        {
            // if (cinfo->err->trace_level > 1)
            //     print_mem_stats(cinfo, pool_id);
        }

        /* If freeing IMAGE pool, close any virtual arrays first */
        if pool_id == JPOOL_IMAGE {
            let mut sptr: jvirt_sarray_ptr;
            let mut bptr: jvirt_barray_ptr;

            sptr = (*mem).virt_sarray_list;
            while !sptr.is_null() {
                if (*sptr).b_s_open != FALSE {	/* there may be no backing store */
                    (*sptr).b_s_open = FALSE;	/* prevent recursive close if error */
                    // (*sptr->b_s_info.close_backing_store)(cinfo, &sptr->b_s_info);
                }
                sptr = (*sptr).next;
            }
            (*mem).virt_sarray_list = null_mut();
            bptr = (*mem).virt_barray_list;
            while !bptr.is_null() {
                if (*bptr).b_s_open != FALSE {	/* there may be no backing store */
                    (*bptr).b_s_open = FALSE;	/* prevent recursive close if error */
                    // (*bptr->b_s_info.close_backing_store)(cinfo, &bptr->b_s_info);
                }
                bptr = (*bptr).next;
            }
            (*mem).virt_barray_list = null_mut();
        }

        /* Release large objects */
        lhdr_ptr = (*mem).large_list[pool_id as usize];
        (*mem).large_list[pool_id as usize] = null_mut();

        while !lhdr_ptr.is_null() {
            let next_lhdr_ptr = (*lhdr_ptr).hdr.next;
            space_freed = (*lhdr_ptr).hdr.bytes_used
                          .wrapping_add((*lhdr_ptr).hdr.bytes_left)
                          .wrapping_add(SIZEOF::<large_pool_struct>());
            jpeg_free_large(cinfo, lhdr_ptr as *const c_void, space_freed);
            (*mem).total_space_allocated -= space_freed as i64;
            lhdr_ptr = next_lhdr_ptr;
        }

        /* Release small objects */
        shdr_ptr = (*mem).small_list[pool_id as usize];
        (*mem).small_list[pool_id as usize] = null_mut();

        while !shdr_ptr.is_null() {
            let next_shdr_ptr = (*shdr_ptr).hdr.next;
            space_freed = (*shdr_ptr).hdr.bytes_used
                          .wrapping_add((*shdr_ptr).hdr.bytes_left)
                          .wrapping_add(SIZEOF::<small_pool_struct>());
            jpeg_free_small(cinfo, shdr_ptr as *const c_void, space_freed);
            (*mem).total_space_allocated -= space_freed as i64;
            shdr_ptr = next_shdr_ptr;
        }
    }
}


/*
 * Close up shop entirely.
 * Note that this cannot be called unless cinfo->mem is non-NULL.
 */

fn self_destruct(cinfo: *const c_void) {
    unsafe {
        let mut pool: c_int;

        /* Close all backing store, release all memory.
         * Releasing pools in reverse order might help avoid fragmentation
         * with some (brain-damaged) malloc libraries.
         */
        pool = (JPOOL_NUMPOOLS as c_int) - 1;
        while pool >= JPOOL_PERMANENT {
            free_pool(cinfo, pool);
            pool -= 1;
        }

        /* Release the memory manager control block too. */
        jpeg_free_small(cinfo, cinfo as *const c_void, SIZEOF::<my_memory_mgr>());
        // cinfo->mem = NULL;		/* ensures I will be called only once */

        jpeg_mem_term(cinfo);		/* system-dependent cleanup */
    }
}


/*
 * Memory manager initialization.
 * When this is called, only the error manager pointer is valid in cinfo!
 */

pub fn jinit_memory_mgr(cinfo: *const c_void) {
    unsafe {
        let mut mem: my_mem_ptr;
        let mut max_to_use: i64;
        let mut pool: c_int;
        let test_mac: usize;

        // cinfo->mem = NULL;		/* for safety if init fails */

        /* Check for configuration errors.
         * SIZEOF(ALIGN_TYPE) should be a power of 2; otherwise, it probably
         * doesn't reflect any real hardware alignment requirement.
         * The test is a little tricky: for X>0, X and X-1 have no one-bits
         * in common if and only if X is a power of 2, ie has only one one-bit.
         * Some compilers may give an "unreachable code" warning here; ignore it.
         */
        if ((SIZEOF::<ALIGN_TYPE>()) & (SIZEOF::<ALIGN_TYPE>().wrapping_sub(1))) != 0 {
            ERREXIT!(cinfo, JERR_BAD_ALIGN_TYPE);
        }
        /* MAX_ALLOC_CHUNK must be representable as type size_t, and must be
         * a multiple of SIZEOF(ALIGN_TYPE).
         * Again, an "unreachable code" warning may be ignored here.
         * But a "constant too large" warning means you need to fix MAX_ALLOC_CHUNK.
         */
        test_mac = MAX_ALLOC_CHUNK as usize;
        if (MAX_ALLOC_CHUNK as usize) != test_mac ||
            ((MAX_ALLOC_CHUNK as usize) % SIZEOF::<ALIGN_TYPE>()) != 0 {
            ERREXIT!(cinfo, JERR_BAD_ALLOC_CHUNK);
        }

        max_to_use = jpeg_mem_init(cinfo); /* system-dependent initialization */

        /* Attempt to allocate memory manager's control block */
        mem = jpeg_get_small(cinfo, SIZEOF::<my_memory_mgr>()) as my_mem_ptr;

        if mem.is_null() {
            jpeg_mem_term(cinfo);	/* system-dependent cleanup */
            ERREXIT1!(cinfo, JERR_OUT_OF_MEMORY, 0);
        }

        /* OK, fill in the method pointers */
        (*mem).pub_.alloc_small = alloc_small as *const c_void;
        (*mem).pub_.alloc_large = alloc_large as *const c_void;
        (*mem).pub_.alloc_sarray = alloc_sarray as *const c_void;
        (*mem).pub_.alloc_barray = alloc_barray as *const c_void;
        (*mem).pub_.request_virt_sarray = request_virt_sarray as *const c_void;
        (*mem).pub_.request_virt_barray = request_virt_barray as *const c_void;
        (*mem).pub_.realize_virt_arrays = realize_virt_arrays as *const c_void;
        (*mem).pub_.access_virt_sarray = access_virt_sarray as *const c_void;
        (*mem).pub_.access_virt_barray = access_virt_barray as *const c_void;
        (*mem).pub_.free_pool = free_pool as *const c_void;
        (*mem).pub_.self_destruct = self_destruct as *const c_void;

        /* Initialize working state */
        (*mem).pub_.max_memory_to_use = max_to_use;

        pool = (JPOOL_NUMPOOLS as c_int) - 1;
        while pool >= JPOOL_PERMANENT {
            (*mem).small_list[pool as usize] = null_mut();
            (*mem).large_list[pool as usize] = null_mut();
            pool -= 1;
        }
        (*mem).virt_sarray_list = null_mut();
        (*mem).virt_barray_list = null_mut();

        (*mem).total_space_allocated = SIZEOF::<my_memory_mgr>() as i64;

        /* Declare ourselves open for business */
        // cinfo->mem = &mem->pub;

        /* Check for an environment variable JPEGMEM; if found, override the
         * default max_memory setting from jpeg_mem_init.  Note that the
         * surrounding application may again override this value.
         * If your system doesn't support getenv(), define NO_GETENV to disable
         * this feature.
         */
        #[cfg(not(feature = "no_getenv"))]
        {
            use std::ffi::CStr;
            use std::env;

            if let Ok(memenv) = env::var("JPEGMEM") {
                let ch = 'x';

                if let Ok(value) = memenv.parse::<i64>() {
                    let mut max_to_use_new = value;
                    if memenv.ends_with('m') || memenv.ends_with('M') {
                        max_to_use_new = max_to_use_new.wrapping_mul(1000);
                    }
                    (*mem).pub_.max_memory_to_use = max_to_use_new.wrapping_mul(1000);
                }
            }
        }
    }
}

// Stub for j_common_ptr to allow compilation
pub struct j_common_ptr {
    err: *mut c_void,
    mem: *mut c_void,
}

#[repr(C)]
pub struct jpeg_error_mgr {
    trace_level: c_int,
}
