////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Handle Pool
// -----------
// The memory pool class is a simple routine for constant time allocation and deallocation
// from a fixed size pool of objects.  The class uses a simple array to hold the actual
// data, a queue for the free list, and a bit field to mark which spots in the array
// are allocated.
//
// In addition to the standard memory pool features, this Handle Pool provides a fast
// iterator, asserts on attempting to access unused data, and a unique ID "handle" for
// the external system to use.
//
//
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::c_int;

// Local stubs for unresolved dependencies
// These would normally be imported from pool_vs.h and ratl_common.h
pub struct array_base<T> {
    phantom: std::marker::PhantomData<T>,
}

pub struct queue_vs<T, const CAPACITY: usize> {
    phantom: std::marker::PhantomData<T>,
}

pub struct bits_base<const CAPACITY: usize> {
    phantom: std::marker::PhantomData<()>,
}

pub struct ratl_base;

pub struct TRatlNew;

pub trait PoolStorageTraits {
    type TStorageTraits;
    type TValue;
    const CAPACITY: usize;
}

pub struct pool_root<T: PoolStorageTraits> {
    mData: array_base<T::TStorageTraits>,
    mFree: queue_vs<i32, { T::CAPACITY }>,
    mUsed: bits_base<{ T::CAPACITY }>,
    mSize: i32,
}

impl<T: PoolStorageTraits> pool_root<T> {
    pub const CAPACITY: usize = T::CAPACITY;

    fn FillFreeList(&mut self) {
        self.mFree.clear();
        let mut i = 0;
        while i < Self::CAPACITY {
            self.mFree.push(i as i32);
            i += 1;
        }
    }

    pub fn new() -> Self {
        let mut pool = pool_root {
            mData: array_base {
                phantom: std::marker::PhantomData,
            },
            mFree: queue_vs {
                phantom: std::marker::PhantomData,
            },
            mUsed: bits_base {
                phantom: std::marker::PhantomData,
            },
            mSize: 0,
        };
        pool.FillFreeList();
        pool
    }

    pub fn clear(&mut self) {
        self.mSize = 0;
        self.mUsed.clear();
        self.mData.clear();
        self.FillFreeList();
    }

    pub fn alloc_index(&mut self) -> usize {
        unimplemented!("pool_root::alloc_index")
    }

    pub fn alloc_index_with(&mut self, _v: &T::TValue) -> usize {
        unimplemented!("pool_root::alloc_index_with")
    }

    pub fn free_index(&mut self, _i: usize) {
        unimplemented!("pool_root::free_index")
    }

    pub fn is_used_index(&self, _i: usize) -> bool {
        unimplemented!("pool_root::is_used_index")
    }

    pub fn value_at_index(&self, _i: usize) -> &T::TValue {
        unimplemented!("pool_root::value_at_index")
    }

    pub fn value_at_index_mut(&mut self, _i: usize) -> &mut T::TValue {
        unimplemented!("pool_root::value_at_index_mut")
    }

    pub fn swap_index(&mut self, _i: usize, _j: usize) {
        unimplemented!("pool_root::swap_index")
    }

    pub fn pointer_to_index(&self, _me: *const T::TValue) -> usize {
        unimplemented!("pool_root::pointer_to_index")
    }

    pub fn at_index(&self, _i: usize) -> usize {
        unimplemented!("pool_root::at_index")
    }

    pub fn at_index_const(&self, _i: usize) -> usize {
        unimplemented!("pool_root::at_index_const")
    }
}

pub struct handle_pool_base<T: PoolStorageTraits> {
    pool: pool_root<T>,
    mHandles: Vec<c_int>,
    mMASK_HANDLE_TO_INDEX: c_int,
    mMASK_NUM_BITS: c_int,
}

// Static global for handle salt in debug builds
#[cfg(debug_assertions)]
pub static mut HandleSaltValue: c_int = 0;

impl<T: PoolStorageTraits> handle_pool_base<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    pub const CAPACITY: usize = T::CAPACITY;

    ////////////////////////////////////////////////////////////////////////////////////
    // IncHandle - private
    ////////////////////////////////////////////////////////////////////////////////////
    fn IncHandle(&mut self, index: usize) {
        self.mHandles[index] += 1 << self.mMASK_NUM_BITS;
        if self.mHandles[index] < 0 {
            // we rolled over
            self.mHandles[index] = index as c_int;  // Reset The ID Counter
            self.mHandles[index] |= 1 << self.mMASK_NUM_BITS;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    //
    // We need a routine to calculate the MASK used to convert a handle to an index and
    // the number of bits needed to shift by.
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        let mut handle_pool = handle_pool_base {
            pool: pool_root::new(),
            mHandles: vec![0; T::CAPACITY],
            mMASK_HANDLE_TO_INDEX: 0,
            mMASK_NUM_BITS: 0,
        };

        handle_pool.mMASK_HANDLE_TO_INDEX = 0;
        handle_pool.mMASK_NUM_BITS = 0;

        let mut value = T::CAPACITY as c_int - 1;
        while value != 0 {
            value >>= 1;

            handle_pool.mMASK_HANDLE_TO_INDEX <<= 1;
            handle_pool.mMASK_HANDLE_TO_INDEX |= 1;
            handle_pool.mMASK_NUM_BITS += 1;
        }
        for i in 0..T::CAPACITY {
            handle_pool.mHandles[i] = i as c_int;  // Reset The ID Counter
            #[cfg(debug_assertions)]
            {
                unsafe {
                    HandleSaltValue += 1;
                    handle_pool.mHandles[i] |= (HandleSaltValue << handle_pool.mMASK_NUM_BITS);
                }
            }
            #[cfg(not(debug_assertions))]
            {
                handle_pool.mHandles[i] |= 1 << handle_pool.mMASK_NUM_BITS;
            }
        }

        handle_pool
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear - Removes all allocation information
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.pool.clear();
        // note that we do not refill the handles cause we want old handles to still be stale
        for i in 0..T::CAPACITY {
            self.IncHandle(i);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_const(&self, handle: c_int) -> &T::TValue {
        assert!(self.is_used(handle));  //typically this is a stale handle (already been freed)
        self.pool.value_at_index((handle & self.mMASK_HANDLE_TO_INDEX) as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_mut(&mut self, i: c_int) -> &mut T::TValue {
        assert!(self.is_used(i));  //typically this is a stale handle (already been freed)
        self.pool.value_at_index_mut((i & self.mMASK_HANDLE_TO_INDEX) as usize)
    }

    pub fn is_used(&self, i: c_int) -> bool {
        let index = (i & self.mMASK_HANDLE_TO_INDEX) as usize;
        if index < self.mHandles.len() && self.mHandles[index] == i {
            self.pool.is_used_index(index)
        } else {
            false
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swap two items based on handle
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn swap(&mut self, i: c_int, j: c_int) {
        assert!(self.is_used(i));  //typically this is a stale handle (already been freed)
        assert!(self.is_used(j));  //typically this is a stale handle (already been freed)
        self.pool.swap_index(self.handle_to_index(i), self.handle_to_index(j));
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator returns a handle
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc(&mut self) -> c_int {
        let index = self.pool.alloc_index();
        self.mHandles[index]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator, with value, returns a handle
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_with(&mut self, v: &T::TValue) -> c_int {
        let index = self.pool.alloc_index_with(v);
        self.mHandles[index]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Deallocator, by index, not something generally needed
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn free_index(&mut self, index: usize) {
        self.pool.free_index(index);
        self.IncHandle(index);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Deallocator, by handle
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn free(&mut self, handle: c_int) {
        assert!(self.is_used(handle));
        self.free_index((handle & self.mMASK_HANDLE_TO_INDEX) as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Deallocator, by pointer
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn free_ptr(&mut self, me: *const T::TValue) {
        self.free_index(self.pool.pointer_to_index(me));
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Convert a handle to a raw index, not generally something you should use
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn handle_to_index(&self, handle: c_int) -> usize {
        assert!(self.is_used(handle));
        (handle & self.mMASK_HANDLE_TO_INDEX) as usize
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Find the handle for a given index, this cannot check for stale handles, so it is ill advised
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn index_to_handle(&self, index: usize) -> c_int {
        assert!(index >= 0 && index < T::CAPACITY && self.pool.is_used_index(index));  //disallowing this on stale handles
        self.mHandles[index]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // converts a T pointer to a handle, generally not something you need, cannot check for stale handles
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pointer_to_handle(&self, me: *const T::TValue) -> c_int {
        self.index_to_handle(self.pool.pointer_to_index(me))
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // converts a T pointer to a handle, generally not something you need, cannot check for stale handles
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pointer_to_handle_TRatlNew(&self, me: *const TRatlNew) -> c_int {
        self.index_to_handle(self.pool.pointer_to_index(me as *const T::TValue))
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Object At handle
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn at(&self, handle: c_int) -> usize {
        assert!(self.is_used(handle));
        self.pool.at_index((handle & self.mMASK_HANDLE_TO_INDEX) as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Object At handle (const version)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn at_const(&self, handle: c_int) -> usize {
        assert!(self.is_used(handle));
        self.pool.at_index_const((handle & self.mMASK_HANDLE_TO_INDEX) as usize)
    }
}

////////////////////////////////////////////////////////////////////////////////////
// Specialized template classes
////////////////////////////////////////////////////////////////////////////////////

pub mod storage {
    use std::marker::PhantomData;
    use super::PoolStorageTraits;

    pub struct value_semantics<T, const CAPACITY: usize> {
        phantom: PhantomData<T>,
    }

    pub struct object_semantics<T, const CAPACITY: usize> {
        phantom: PhantomData<T>,
    }

    pub struct virtual_semantics<T, const CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
        phantom: PhantomData<T>,
    }

    impl<T: Sized, const CAPACITY: usize> PoolStorageTraits for value_semantics<T, CAPACITY> {
        type TStorageTraits = super::array_base<T>;
        type TValue = T;
        const CAPACITY: usize = CAPACITY;
    }

    impl<T: Sized, const CAPACITY: usize> PoolStorageTraits for object_semantics<T, CAPACITY> {
        type TStorageTraits = super::array_base<T>;
        type TValue = T;
        const CAPACITY: usize = CAPACITY;
    }

    impl<T: Sized, const CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> PoolStorageTraits for virtual_semantics<T, CAPACITY, ARG_MAX_CLASS_SIZE> {
        type TStorageTraits = super::array_base<T>;
        type TValue = T;
        const CAPACITY: usize = CAPACITY;
    }
}

pub struct handle_pool_vs<T, const ARG_CAPACITY: usize> {
    base: handle_pool_base<storage::value_semantics<T, ARG_CAPACITY>>,
}

impl<T: Sized, const ARG_CAPACITY: usize> handle_pool_vs<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    pub fn new() -> Self {
        handle_pool_vs {
            base: handle_pool_base::new(),
        }
    }
}

pub struct handle_pool_os<T, const ARG_CAPACITY: usize> {
    base: handle_pool_base<storage::object_semantics<T, ARG_CAPACITY>>,
}

impl<T: Sized, const ARG_CAPACITY: usize> handle_pool_os<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    pub fn new() -> Self {
        handle_pool_os {
            base: handle_pool_base::new(),
        }
    }
}

pub struct handle_pool_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    base: handle_pool_base<storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

impl<T: Sized, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> handle_pool_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    pub const CAPACITY: usize = ARG_CAPACITY;
    pub const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;

    pub fn new() -> Self {
        handle_pool_is {
            base: handle_pool_base::new(),
        }
    }
}
