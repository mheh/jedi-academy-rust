////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Heap
// ------
//
//
//
//
// TODO:
//
//
// NOTES:
//
//
////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::c_int;

////////////////////////////////////////////////////////////////////////////////////////
// Includes
////////////////////////////////////////////////////////////////////////////////////////
// ratl_common.h equivalent would be imported here
// namespace ratl equivalent: module scope

////////////////////////////////////////////////////////////////////////////////////////
// Local stubs for RATL dependencies
////////////////////////////////////////////////////////////////////////////////////////

/// Trait representing RATL array base storage - dependency from ratl library
pub trait RatlArrayBase<T>: Sized {
    fn construct(&mut self, index: usize, value: T);
    fn construct_default(&mut self, index: usize);
    fn destruct(&mut self, index: usize);
    fn swap(&mut self, a: usize, b: usize);
    fn clear(&mut self);
    fn get(&self, index: usize) -> &T;
    fn get_mut(&mut self, index: usize) -> &mut T;
}

/// Base type for RATL - currently a stub
pub struct RatlBase;

/// Stub for TRatlNew type used in alloc_raw
pub type TRatlNew = u8;

/// Local stub for array_base storage trait types
pub mod storage {
    use super::RatlArrayBase;

    /// Trait representing storage with value semantics
    pub trait ValueSemantics {
        type TValue;
        const CAPACITY: usize;
    }

    /// Trait representing storage with object semantics
    pub trait ObjectSemantics {
        type TValue;
        const CAPACITY: usize;
    }

    /// Trait representing storage with virtual semantics
    pub trait VirtualSemantics {
        type TValue;
        const CAPACITY: usize;
        const MAX_CLASS_SIZE: usize;
    }

    /// Concrete implementation stub for value_semantics
    pub struct value_semantics<T, const ARG_CAPACITY: usize> {
        _phantom: std::marker::PhantomData<T>,
    }

    /// Concrete implementation stub for object_semantics
    pub struct object_semantics<T, const ARG_CAPACITY: usize> {
        _phantom: std::marker::PhantomData<T>,
    }

    /// Concrete implementation stub for virtual_semantics
    pub struct virtual_semantics<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
        _phantom: std::marker::PhantomData<T>,
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Heap Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct heap_base<T, Storage>
where
    T: Ord + Clone,
    Storage: RatlArrayBase<T>,
{
    mData: Storage,          // The Memory
    mPush: i32,              // Address Of Next Add Location
    _phantom: std::marker::PhantomData<T>,
}

impl<T, Storage> heap_base<T, Storage>
where
    T: Ord + Clone,
    Storage: RatlArrayBase<T>,
{
    const CAPACITY: usize = 256; // Default; would be parameterized from Storage trait

    ////////////////////////////////////////////////////////////////////////////////////
    // Returns The Location Of Node (i)'s Parent Node (The Parent Node Of Zero Is Zero)
    ////////////////////////////////////////////////////////////////////////////////////
    fn parent(i: i32) -> i32 {
        (i - 1) / 2
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Returns The Location Of Node (i)'s Left Child (The Child Of A Leaf Is The Leaf)
    ////////////////////////////////////////////////////////////////////////////////////
    fn left(i: i32) -> i32 {
        (2 * i) + 1
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Returns The Location Of Node (i)'s Right Child (The Child Of A Leaf Is The Leaf)
    ////////////////////////////////////////////////////////////////////////////////////
    fn right(i: i32) -> i32 {
        (2 * i) + 2
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Returns The Location Of Largest Child Of Node (i)
    ////////////////////////////////////////////////////////////////////////////////////
    fn largest_child(&self, i: i32) -> i32 {
        if Self::left(i) < self.mPush {
            if Self::right(i) < self.mPush {
                let right_idx = Self::right(i) as usize;
                let left_idx = Self::left(i) as usize;
                return if self.mData.get(right_idx) < self.mData.get(left_idx) {
                    Self::left(i)
                } else {
                    Self::right(i)
                };
            }
            return Self::left(i); // Node i only has a left child, so by default it is the biggest
        }
        i // Node i is a leaf, so just return it
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swaps Two Element Locations
    ////////////////////////////////////////////////////////////////////////////////////
    fn swap(&mut self, a: i32, b: i32) {
        if a == b {
            return;
        }
        assert!(a >= 0 && b >= 0 && (a as usize) < Self::CAPACITY && (b as usize) < Self::CAPACITY);
        self.mData.swap(a as usize, b as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swaps The Data Up The Heap Until It Reaches A Valid Location
    ////////////////////////////////////////////////////////////////////////////////////
    fn reheapify_upward(&mut self, mut pos: i32) {
        while pos != 0 && self.mData.get(Self::parent(pos) as usize) < self.mData.get(pos as usize) {
            self.swap(Self::parent(pos), pos);
            pos = Self::parent(pos);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swaps The Data Down The Heap Until It Reaches A Valid Location
    ////////////////////////////////////////////////////////////////////////////////////
    fn reheapify_downward(&mut self, mut pos: i32) {
        let mut largest_child = self.largest_child(pos);
        while largest_child != pos && self.mData.get(pos as usize) < self.mData.get(largest_child as usize) {
            self.swap(largest_child, pos);
            pos = largest_child;
            largest_child = self.largest_child(pos);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Validate Will Run Through The Heap And Make Sure The Top Element Is Smallest
    ////////////////////////////////////////////////////////////////////////////////////
    fn valid(&self) -> bool {
        for i in 1..self.mPush {
            if self.mData.get(0) < self.mData.get(i as usize) {
                return false;
            }
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new(storage: Storage) -> Self {
        heap_base {
            mData: storage,
            mPush: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get The Size (The Difference Between The Push And Pop "Pointers")
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> i32 {
        self.mPush
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If The Size Is Zero
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If The Size Is Full
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        self.size() == Self::CAPACITY as i32
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Empty Out The Entire Heap
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mPush = 0;
        self.mData.clear();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get The Data Value At The Top Of The Heap
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn top(&self) -> &T {
        assert!(self.mPush > 0); // Don't Try To Look At This If There Is Nothing In Here
        self.mData.get(0)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add A Value To The Queue
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push(&mut self, nValue: T) {
        assert!(self.size() < Self::CAPACITY as i32);

        // Add It
        //--------
        self.mData.construct(self.mPush as usize, nValue);

        // Fix Possible Heap Inconsistancies
        //-----------------------------------
        self.reheapify_upward(self.mPush);

        self.mPush += 1;
        assert!(self.valid());
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Alloc A Value, call push_alloced to add
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc(&mut self) -> &mut T {
        assert!(self.size() < Self::CAPACITY as i32);

        // Add It
        //--------
        self.mData.construct_default(self.mPush as usize);

        self.mData.get_mut(self.mPush as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Alloc A Raw Value for placement new, call push_alloced to add
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn alloc_raw(&mut self) -> *mut TRatlNew {
        assert!(self.size() < Self::CAPACITY as i32);

        // Returns raw pointer to allocation space
        self.mData.get_mut(self.mPush as usize) as *mut T as *mut TRatlNew
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add A Value To The Queue, after filling an alloced slot
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_alloced(&mut self) {
        assert!(self.size() < Self::CAPACITY as i32);
        // Fix Possible Heap Inconsistancies
        //-----------------------------------
        self.reheapify_upward(self.mPush);

        self.mPush += 1;
        assert!(self.valid());
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Remove A Value From The Queue
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pop(&mut self) {
        assert!(self.size() > 0);

        self.mPush -= 1;

        // Swap The Lowest Element Up To The Spot We Just "Erased"
        //---------------------------------------------------------
        self.swap(0, self.mPush);
        self.mData.destruct(self.mPush as usize);

        // Fix Possible Heap Inconsistancies
        //-----------------------------------
        self.reheapify_downward(0);
        assert!(self.valid());
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// heap_vs: Heap with Value Semantics
////////////////////////////////////////////////////////////////////////////////////////
pub struct heap_vs<T: Ord + Clone, const ARG_CAPACITY: usize>
where
    T: Ord + Clone,
{
    base: heap_base<T, storage::value_semantics<T, ARG_CAPACITY>>,
}

impl<T: Ord + Clone, const ARG_CAPACITY: usize> heap_vs<T, ARG_CAPACITY> {
    const CAPACITY: usize = ARG_CAPACITY;

    pub fn new(storage: storage::value_semantics<T, ARG_CAPACITY>) -> Self {
        heap_vs {
            base: heap_base::new(storage),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// heap_os: Heap with Object Semantics
////////////////////////////////////////////////////////////////////////////////////////
pub struct heap_os<T: Ord + Clone, const ARG_CAPACITY: usize>
where
    T: Ord + Clone,
{
    base: heap_base<T, storage::object_semantics<T, ARG_CAPACITY>>,
}

impl<T: Ord + Clone, const ARG_CAPACITY: usize> heap_os<T, ARG_CAPACITY> {
    const CAPACITY: usize = ARG_CAPACITY;

    pub fn new(storage: storage::object_semantics<T, ARG_CAPACITY>) -> Self {
        heap_os {
            base: heap_base::new(storage),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// heap_is: Heap with Virtual Semantics
////////////////////////////////////////////////////////////////////////////////////////
pub struct heap_is<T: Ord + Clone, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize>
where
    T: Ord + Clone,
{
    base: heap_base<T, storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

impl<T: Ord + Clone, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> heap_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    const CAPACITY: usize = ARG_CAPACITY;
    const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;

    pub fn new(storage: storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>) -> Self {
        heap_is {
            base: heap_base::new(storage),
        }
    }
}
