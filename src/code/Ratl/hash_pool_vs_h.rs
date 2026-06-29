////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Hash Pool
// ---------
// The hash pool stores raw data of variable size.  It uses a hash table to check for
// redundant data, and upon finding any, will return the existing handle.  Otherwise
// it copies the data to memory and returns a new handle.
//
//
// NOTES:
//
//
//

use core::ffi::{c_char, c_void, c_int};
use core::ptr::{addr_of, addr_of_mut};

// Memory utility functions (local stubs for ratl::mem namespace)
mod mem {
    use core::ffi::c_void;
    use core::ptr;

    pub unsafe fn eql(a: *const c_void, b: *const c_void, size: usize) -> bool {
        // Compare memory blocks
        let a_bytes = a as *const u8;
        let b_bytes = b as *const u8;
        for i in 0..size {
            if *a_bytes.add(i) != *b_bytes.add(i) {
                return false;
            }
        }
        true
    }

    pub unsafe fn cpy(dest: *mut c_void, src: *const c_void, size: usize) {
        ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, size);
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Hash Pool
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
pub struct hash_pool<const SIZE: usize, const SIZE_HANDLES: usize> {
    mHandles: [c_int; SIZE_HANDLES],					// each handle holds the start index of it's data

    mDataAlloc: c_int,								// where the next chunck of data will go
    mData: [c_char; SIZE],

    #[cfg(debug_assertions)]
    mFinds: c_int,									// counts how many total finds have run
    #[cfg(debug_assertions)]
    mCurrentCollisions: c_int,						// counts how many collisions on the last find
    #[cfg(debug_assertions)]
    mTotalCollisions: c_int,						// counts the total number of collisions
    #[cfg(debug_assertions)]
    mTotalAllocs: c_int,
}

impl<const SIZE: usize, const SIZE_HANDLES: usize> hash_pool<SIZE, SIZE_HANDLES> {

    ////////////////////////////////////////////////////////////////////////////////////
    // This function searches for a handle which already stores the data (assuming the
    // handle is a hash within range SIZE_HANDLES).
    //
    // If it failes, it returns false, and the handle passed in points to the next
    // free slot.
    ////////////////////////////////////////////////////////////////////////////////////
    fn find_existing(&mut self, handle: &mut c_int, data: *const c_void, datasize: c_int) -> bool
    {
        #[cfg(debug_assertions)]
        {
            self.mFinds += 1;
            self.mCurrentCollisions = 0;
        }

        unsafe {
            while self.mHandles[*handle as usize] != 0 {					// So long as a handle exists there
                if mem::eql(
                    addr_of!(self.mData[self.mHandles[*handle as usize] as usize]) as *const c_void,
                    data,
                    datasize as usize
                ) {
                    return true;						// found
                }
                *handle = ((*handle) + 1) & ((SIZE_HANDLES as c_int) - 1);		// incriment the handle

                #[cfg(debug_assertions)]
                {
                    self.mCurrentCollisions += 1;
                    self.mTotalCollisions += 1;

                    //assert(mCurrentCollisions < 16);		// If We Had 16+ Collisions, Hash May Be Inefficient.
                    // Evaluate SIZE and SIZEHANDLES
                }
            }
        }
        false								// failed to find
    }



    ////////////////////////////////////////////////////////////////////////////////////
    // A simple hash function for the range of [0, SIZE_HANDLES]
    ////////////////////////////////////////////////////////////////////////////////////
    fn hash(&self, data: *const c_void, datasize: c_int) -> c_int
    {
        let mut h: c_int = 0;
        unsafe {
            for i in 0..datasize {
                let byte_ptr = data as *const c_char;
                let byte_val = *byte_ptr.add(i as usize) as c_int;
                h += byte_val * (i + 119);		// 119.  Prime Number?
            }
        }
        h &= (SIZE_HANDLES as c_int) - 1;						// zero out bits beyoned SIZE_HANDLES
        h
    }

    pub fn new() -> Self {
        let mut pool = hash_pool {
            mHandles: [0; SIZE_HANDLES],
            mDataAlloc: 0,
            mData: [0; SIZE],
            #[cfg(debug_assertions)]
            mFinds: 0,
            #[cfg(debug_assertions)]
            mCurrentCollisions: 0,
            #[cfg(debug_assertions)]
            mTotalCollisions: 0,
            #[cfg(debug_assertions)]
            mTotalAllocs: 0,
        };
        pool.clear();
        pool
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Number Of Bytes Allocated
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> c_int
    {
        self.mDataAlloc
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If This Memory Pool Is Empty
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool
    {
        self.mDataAlloc == 1
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If This Memory Pool Has Enough Space Left For (minimum) Bytes
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self, minimum: c_int) -> bool
    {
        ((SIZE as c_int) - self.mDataAlloc) < minimum
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear - Removes all allocation information - Note!  DOES NOT CLEAR MEMORY
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self)
    {
        self.mData[0] = 0;
        self.mDataAlloc = 1;
        for i in 0..SIZE_HANDLES {
            self.mHandles[i] = 0;
        }

        #[cfg(debug_assertions)]
        {
            self.mFinds = 0;
            self.mCurrentCollisions = 0;
            self.mTotalCollisions = 0;
            self.mTotalAllocs = 0;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // This is the primary functionality of the hash pool.  It will search for existing
    // data of the same size, and failing to find any, it will append the data to the
    // memory.
    //
    // In both cases, it gives you a handle to look up the data later.
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_handle(&mut self, data: *const c_void, datasize: c_int) -> c_int
    {
        let mut handle = self.hash(data, datasize);				// Initialize Our Handle By Hash Fcn
        if !self.find_existing(&mut handle, data, datasize)
        {
            assert!((self.mDataAlloc + datasize) < (SIZE as c_int));			// Is There Enough Memory?

            #[cfg(debug_assertions)]
            {
                self.mTotalAllocs += 1;
            }

            unsafe {
                mem::cpy(
                    addr_of_mut!(self.mData[self.mDataAlloc as usize]) as *mut c_void,
                    data,
                    datasize as usize
                );
            }
            self.mHandles[handle as usize] = self.mDataAlloc;				// Mark Memory In Hash Tbl
            self.mDataAlloc += datasize;						// Adjust Next Alloc Location
        }
        handle									// Return The Hash Tbl handleess
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Access Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn index(&self, handle: c_int) -> *const c_void
    {
        assert!(handle >= 0 && handle < (SIZE_HANDLES as c_int));

        unsafe {
            addr_of!(self.mData[self.mHandles[handle as usize] as usize]) as *const c_void
        }
    }


    #[cfg(debug_assertions)]
    pub fn average_collisions(&self) -> f32 {
        (self.mTotalCollisions as f32) / (self.mFinds as f32)
    }

    #[cfg(debug_assertions)]
    pub fn total_allocs(&self) -> c_int {
        self.mTotalAllocs
    }

    #[cfg(debug_assertions)]
    pub fn total_finds(&self) -> c_int {
        self.mFinds
    }

    #[cfg(debug_assertions)]
    pub fn total_collisions(&self) -> c_int {
        self.mTotalCollisions
    }

}
