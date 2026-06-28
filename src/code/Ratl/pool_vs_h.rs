////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Memory Pool
// -----------
// The memory pool class is a simple routine for constant time allocation and deallocation
// from a fixed size pool of objects.  The class uses a simple array to hold the actual
// data, a queue for the free list, and a bit field to mark which spots in the array
// are allocated.
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

// Local stubs for unresolved dependencies
// These would normally be imported from ratl_common.h and queue_vs.h
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

// fixme, this could be made more efficient by keepingtrack of the highest slot ever used
// then there is no need to fill the free list
pub struct pool_root<T: PoolStorageTraits> {
    mData: array_base<T::TStorageTraits>,
    mFree: queue_vs<i32, { T::CAPACITY }>,
    mUsed: bits_base<{ T::CAPACITY }>,
    mSize: i32,
}

pub trait PoolStorageTraits {
    type TStorageTraits;
    type TValue;
    const CAPACITY: usize;
}

impl<T: PoolStorageTraits> pool_root<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    pub const CAPACITY: usize = T::CAPACITY;

    ////////////////////////////////////////////////////////////////////////////////////
    // Data
    ////////////////////////////////////////////////////////////////////////////////////

    fn FillFreeList(&mut self) {
        self.mFree.clear();
        let mut i = 0;
        while i < Self::CAPACITY {
            self.mFree.push(i as i32);
            i += 1;
        }
    }

    fn alloc_low(&mut self) -> i32 {
        assert!(self.mSize < Self::CAPACITY as i32);
        assert!(!self.mUsed[self.mFree.top() as usize]);

        let NextIndex = self.mFree.top(); // Get The First Available Location

        self.mUsed.set_bit(NextIndex as usize);
        self.mFree.pop();
        self.mSize += 1;
        NextIndex
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////
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

    ////////////////////////////////////////////////////////////////////////////////////
    // The Number Of Objects Allocated
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> i32 {
        self.mSize
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If This Memory Pool Is Empty
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        self.mSize == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If This Memory Pool Is Full
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        self.mSize == Self::CAPACITY as i32
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn value_at_index(&self, i: usize) -> &T::TValue {
        assert!(self.mUsed[i]);
        &self.mData[i]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn value_at_index_mut(&mut self, i: usize) -> &mut T::TValue {
        assert!(self.mUsed[i]);
        &mut self.mData[i]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear - Removes all allocation information
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mSize = 0;
        self.mUsed.clear();
        self.mData.clear();
        self.FillFreeList();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check If An Index Has Been Allocated
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn is_used_index(&self, i: usize) -> bool {
        self.mUsed[i]
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Convert a pointer back to an index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pointer_to_index(&self, me: *const T::TValue) -> i32 {
        assert!(self.mSize > 0);

        let index = self.mData.pointer_to_index(me);

        assert!(index >= 0 && index < Self::CAPACITY as i32);
        assert!(self.mUsed[index as usize]); // I am disallowing obtaining the index of a freed item

        index
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Convert a pointer back to an index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pointer_to_index_TRatlNew(&self, me: *const TRatlNew) -> i32 {
        assert!(self.mSize > 0);

        let index = self.mData.pointer_to_index(me as *const T::TValue);

        assert!(index >= 0 && index < Self::CAPACITY as i32);
        assert!(self.mUsed[index as usize]); // I am disallowing obtaining the index of a freed item

        index
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swap two items based on index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn swap_index(&mut self, i: i32, j: i32) {
        assert!(i >= 0 && i < Self::CAPACITY as i32);
        assert!(j >= 0 && j < Self::CAPACITY as i32);
        self.mData.swap(i as usize, j as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_index(&mut self) -> i32 {
        let NextIndex = self.alloc_low();
        self.mData.construct(NextIndex as usize);
        NextIndex
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_index_with_value(&mut self, v: &T::TValue) -> i32 {
        let NextIndex = self.alloc_low();
        self.mData.construct_with_value(NextIndex as usize, v);
        NextIndex
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_raw(&mut self) -> *mut TRatlNew {
        let NextIndex = self.alloc_low();
        self.mData.alloc_raw(NextIndex as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Deallocator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn free_index(&mut self, i: i32) {
        assert!(self.mSize > 0);
        assert!(i >= 0 && i < Self::CAPACITY as i32);
        assert!(self.mUsed[i as usize]);

        self.mData.destruct(i as usize);

        self.mUsed.clear_bit(i as usize);
        self.mFree.push(i);
        self.mSize -= 1;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Deallocator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn free(&mut self, me: *const T::TValue) {
        self.free_index(self.pointer_to_index(me));
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin(&self) -> iterator<T> {
        let idx = self.mUsed.next_bit(0);
        iterator {
            mIndex: idx,
            mOwner: self as *const _,
        } // Find The First Allocated
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Object At index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn at_index_iter(&self, index: i32) -> iterator<T> {
        assert!(self.mUsed[index as usize]); // disallow iterators to non alloced things
        iterator {
            mIndex: index,
            mOwner: self as *const _,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The End Of The Memroy (One Step Beyond)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end(&self) -> iterator<T> {
        iterator {
            mIndex: Self::CAPACITY as i32,
            mOwner: self as *const _,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The First Allocated Memory Block
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_const(&self) -> const_iterator<T> {
        let idx = self.mUsed.next_bit(0);
        const_iterator {
            mIndex: idx,
            mOwner: self as *const _,
        } // Find The First Allocated
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Object At index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn at_index_const_iter(&self, index: i32) -> const_iterator<T> {
        assert!(self.mUsed[index as usize]); // disallow iterators to non alloced things
        const_iterator {
            mIndex: index,
            mOwner: self as *const _,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The End Of The Memroy (One Step Beyond)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end_const(&self) -> const_iterator<T> {
        const_iterator {
            mIndex: Self::CAPACITY as i32,
            mOwner: self as *const _,
        }
    }

    pub fn verify_alloc<CAST_TO>(&self, p: *const CAST_TO) -> *const CAST_TO {
        self.mData.verify_alloc(p)
    }
}

////////////////////////////////////////////////////////////////////////////////////
// Iterator
////////////////////////////////////////////////////////////////////////////////////
pub struct iterator<T: PoolStorageTraits> {
    mIndex: i32,
    mOwner: *const pool_root<T>,
}

impl<T: PoolStorageTraits> iterator<T> {
    // Constructors
    //--------------
    pub fn new() -> Self {
        iterator {
            mIndex: 0,
            mOwner: std::ptr::null(),
        }
    }

    pub fn with_owner(p: *const pool_root<T>, index: i32) -> Self {
        iterator {
            mIndex: index,
            mOwner: p,
        }
    }

    pub fn copy(t: &iterator<T>) -> Self {
        iterator {
            mOwner: t.mOwner,
            mIndex: t.mIndex,
        }
    }

    // Assignment Operator
    //---------------------
    pub fn assign(&mut self, t: &iterator<T>) {
        self.mOwner = t.mOwner;
        self.mIndex = t.mIndex;
    }

    // Equality Operators
    //--------------------
    pub fn not_equal(&self, t: &iterator<T>) -> bool {
        assert!(!self.mOwner.is_null() && (self.mOwner == t.mOwner));
        self.mIndex != t.mIndex
    }

    pub fn equal(&self, t: &iterator<T>) -> bool {
        assert!(!self.mOwner.is_null() && (self.mOwner == t.mOwner));
        self.mIndex == t.mIndex
    }

    // Dereference Operators
    //----------------------
    pub fn deref(&self) -> &T::TValue {
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        unsafe { &(*self.mOwner).mData[self.mIndex as usize] }
    }

    pub fn arrow(&self) -> *const T::TValue {
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        unsafe { &(*self.mOwner).mData[self.mIndex as usize] as *const _ }
    }

    // Handle & Index Access
    //-----------------------
    pub fn index(&self) -> i32 {
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        self.mIndex
    }

    // Inc Operator
    //-------------
    pub fn postfix_increment(&mut self) -> iterator<T> {
        // postfix
        assert!(self.mIndex >= 0 && self.mIndex < T::CAPACITY as i32); // this typically means you did end()++
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );

        let ret = iterator {
            mIndex: self.mIndex,
            mOwner: self.mOwner,
        };
        self.mIndex = unsafe { (*self.mOwner).mUsed.next_bit((self.mIndex + 1) as usize) as i32 };
        ret
    }

    // Inc Operator
    //-------------
    pub fn prefix_increment(&mut self) -> i32 {
        // prefix
        assert!(self.mIndex >= 0 && self.mIndex < T::CAPACITY as i32); // this typically means you did end()++
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        self.mIndex = unsafe { (*self.mOwner).mUsed.next_bit((self.mIndex + 1) as usize) as i32 };
        self.mIndex
    }
}

////////////////////////////////////////////////////////////////////////////////////
// Iterator
////////////////////////////////////////////////////////////////////////////////////
pub struct const_iterator<T: PoolStorageTraits> {
    mIndex: i32,
    mOwner: *const pool_root<T>,
}

impl<T: PoolStorageTraits> const_iterator<T> {
    // Constructors
    //--------------
    pub fn new() -> Self {
        const_iterator {
            mIndex: 0,
            mOwner: std::ptr::null(),
        }
    }

    pub fn with_owner(p: *const pool_root<T>, index: i32) -> Self {
        const_iterator {
            mIndex: index,
            mOwner: p,
        }
    }

    pub fn from_iterator(t: &iterator<T>) -> Self {
        const_iterator {
            mIndex: t.mIndex,
            mOwner: t.mOwner,
        }
    }

    pub fn copy(t: &const_iterator<T>) -> Self {
        const_iterator {
            mIndex: t.mIndex,
            mOwner: t.mOwner,
        }
    }

    // Equality Operators
    //--------------------
    pub fn not_equal(&self, t: &const_iterator<T>) -> bool {
        assert!(!self.mOwner.is_null() && (self.mOwner == t.mOwner));
        self.mIndex != t.mIndex
    }

    pub fn equal(&self, t: &const_iterator<T>) -> bool {
        assert!(!self.mOwner.is_null() && (self.mOwner == t.mOwner));
        self.mIndex == t.mIndex
    }

    pub fn not_equal_iter(&self, t: &iterator<T>) -> bool {
        assert!(!self.mOwner.is_null() && (self.mOwner == t.mOwner));
        self.mIndex != t.mIndex
    }

    pub fn equal_iter(&self, t: &iterator<T>) -> bool {
        assert!(!self.mOwner.is_null() && (self.mOwner == t.mOwner));
        self.mIndex == t.mIndex
    }

    // Dereference Operators
    //----------------------
    pub fn deref(&self) -> &T::TValue {
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        unsafe { &(*self.mOwner).mData[self.mIndex as usize] }
    }

    pub fn arrow(&self) -> *const T::TValue {
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        unsafe { &(*self.mOwner).mData[self.mIndex as usize] as *const _ }
    }

    // Handle & Index Access
    //-----------------------
    pub fn index(&self) -> i32 {
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        self.mIndex
    }

    // Inc Operator
    //-------------
    pub fn postfix_increment(&mut self) -> const_iterator<T> {
        // postfix
        assert!(self.mIndex >= 0 && self.mIndex < T::CAPACITY as i32); // this typically means you did end()++
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );

        let ret = const_iterator {
            mIndex: self.mIndex,
            mOwner: self.mOwner,
        };
        self.mIndex = unsafe { (*self.mOwner).mUsed.next_bit((self.mIndex + 1) as usize) as i32 };
        ret
    }

    // Inc Operator
    //-------------
    pub fn prefix_increment(&mut self) -> i32 {
        // prefix
        assert!(self.mIndex >= 0 && self.mIndex < T::CAPACITY as i32); // this typically means you did end()++
        assert!(
            !self.mOwner.is_null() && unsafe { (*self.mOwner).is_used_index(self.mIndex as usize) }
        );
        self.mIndex = unsafe { (*self.mOwner).mUsed.next_bit((self.mIndex + 1) as usize) as i32 };
        self.mIndex
    }
}

/*
pool_base, base class for the pools

operations:

size()
empty()
full()
clear()							op[]
at_index()						op[]
at_index() const
index pointer_to_index(ptr)
index alloc_index()				alloc()
index alloc_index(ref)			alloc()
ptr alloc_raw()
free_index(index)
free(ptr)
is_used_index(index)

*/

pub struct pool_base<T: PoolStorageTraits> {
    root: pool_root<T>,
}

impl<T: PoolStorageTraits> pool_base<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn index_const(&self, i: usize) -> &T::TValue {
        self.root.value_at_index(i)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Accessor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn index_mut(&mut self, i: usize) -> &mut T::TValue {
        self.root.value_at_index_mut(i)
    }

    pub fn is_used(&self, i: usize) -> bool {
        self.root.is_used_index(i)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swap two items based on index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn swap(&mut self, i: i32, j: i32) {
        self.root.swap_index(i, j);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator	returns an index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc(&mut self) -> i32 {
        self.root.alloc_index()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Allocator	returns an index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_with_value(&mut self, v: &T::TValue) -> i32 {
        self.root.alloc_index_with_value(v)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Deallocator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn free(&mut self, i: i32) {
        self.root.free_index(i);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Object At index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn at(&self, index: i32) -> iterator<T> {
        self.root.at_index_iter(index)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get An Iterator To The Object At index
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn at_const(&self, index: i32) -> const_iterator<T> {
        self.root.at_index_const_iter(index)
    }
}

pub struct pool_vs<T, const ARG_CAPACITY: usize> {
    base: pool_base<pool_vs_storage<T, ARG_CAPACITY>>,
}

pub struct pool_vs_storage<T, const ARG_CAPACITY: usize> {
    phantom: std::marker::PhantomData<T>,
}

impl<T, const ARG_CAPACITY: usize> PoolStorageTraits for pool_vs_storage<T, ARG_CAPACITY> {
    type TStorageTraits = pool_vs_storage<T, ARG_CAPACITY>;
    type TValue = T;
    const CAPACITY: usize = ARG_CAPACITY;
}

pub struct pool_os<T, const ARG_CAPACITY: usize> {
    base: pool_base<pool_os_storage<T, ARG_CAPACITY>>,
}

pub struct pool_os_storage<T, const ARG_CAPACITY: usize> {
    phantom: std::marker::PhantomData<T>,
}

impl<T, const ARG_CAPACITY: usize> PoolStorageTraits for pool_os_storage<T, ARG_CAPACITY> {
    type TStorageTraits = pool_os_storage<T, ARG_CAPACITY>;
    type TValue = T;
    const CAPACITY: usize = ARG_CAPACITY;
}

pub struct pool_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    base: pool_base<pool_is_storage<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

pub struct pool_is_storage<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    phantom: std::marker::PhantomData<T>,
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> PoolStorageTraits
    for pool_is_storage<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
{
    type TStorageTraits = pool_is_storage<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>;
    type TValue = T;
    const CAPACITY: usize = ARG_CAPACITY;
}
