////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Vector
// ------
// The vector class is a simple addition to the array.  It supports some useful additions
// like sort and binary search, as well as keeping track of the number of objects
// contained within.
//
//
//
//
//
// NOTES:
//
//
////////////////////////////////////////////////////////////////////////////////////////
// Includes
#![allow(non_snake_case)]

use core::marker::PhantomData;
use core::ptr::addr_of;
use core::ptr::addr_of_mut;

// Forward declare or stub necessary dependencies
// These would normally come from ratl_common.h and ratl/storage.h
// The actual implementations are expected to be provided by the module system

pub trait TStorageTraits: Sized {
    type TValue: Clone + Copy + PartialOrd;
    const CAPACITY: i32;

    // Required methods for storage operations
    fn raw_array(&self) -> *const Self::TValue;
    fn raw_array_mut(&mut self) -> *mut Self::TValue;
    fn construct(&mut self, index: usize);
    fn construct_with(&mut self, index: usize, value: Self::TValue);
    fn destruct(&mut self, index: usize);
    fn swap(&mut self, i: usize, j: usize);
    fn alloc_raw(&mut self, index: usize) -> *mut u8;
    fn verify_alloc<CAST_TO>(&self, p: *mut CAST_TO) -> *mut CAST_TO;
    fn index(&self, index: usize) -> Self::TValue;
    fn index_mut(&mut self, index: usize) -> *mut Self::TValue;
    fn clear(&mut self);
}

pub trait ratl_base {}

pub struct array_base<T: TStorageTraits> {
    _phantom: PhantomData<T>,
}

impl<T: TStorageTraits> array_base<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        // Stub
    }

    pub fn construct(&mut self, _index: usize) {
        // Stub
    }

    pub fn construct_with(&mut self, _index: usize, _value: T::TValue) {
        // Stub
    }

    pub fn destruct(&mut self, _index: usize) {
        // Stub
    }

    pub fn swap(&mut self, _i: usize, _j: usize) {
        // Stub
    }

    pub fn alloc_raw(&mut self, _index: usize) -> *mut u8 {
        std::ptr::null_mut()
    }

    pub fn verify_alloc<CAST_TO>(&self, p: *mut CAST_TO) -> *mut CAST_TO {
        p
    }
}

pub mod storage {
    use super::*;

    pub struct value_semantics<T, const ARG_CAPACITY: i32> {
        _phantom: PhantomData<T>,
    }

    pub struct object_semantics<T, const ARG_CAPACITY: i32> {
        _phantom: PhantomData<T>,
    }

    pub struct virtual_semantics<T, const ARG_CAPACITY: i32, const ARG_MAX_CLASS_SIZE: i32> {
        _phantom: PhantomData<T>,
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Vector Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct vector_base<T: TStorageTraits> {
    mArray: array_base<T>,  // The Memory
    mSize: i32,
}

impl<T: TStorageTraits> vector_base<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    pub const CAPACITY: i32 = T::CAPACITY;

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        Self {
            mArray: array_base::new(),
            mSize: 0,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn from_vector(B: &vector_base<T>) -> Self {
        let mut new_vec = Self {
            mArray: array_base::new(),
            mSize: 0,
        };
        for i in 0..B.size() {
            new_vec.mArray.construct_with(i as usize, B.mArray.index(i as usize));
        }
        new_vec.mSize = B.mSize;  // NOTE: Original C++ code has bug using val.mSize instead of B.mSize
        new_vec
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // How Many Objects Can Be Added?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn capacity(&self) -> i32 {
        assert!(self.mSize >= 0 && self.mSize <= T::CAPACITY);
        T::CAPACITY
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // How Many Objects Have Been Added To This Vector?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> i32 {
        assert!(self.mSize >= 0 && self.mSize <= T::CAPACITY);
        self.mSize
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Have Any Objects Have Been Added To This Vector?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        assert!(self.mSize >= 0 && self.mSize <= T::CAPACITY);
        self.mSize == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Have Any Objects Have Been Added To This Vector?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        assert!(self.mSize >= 0 && self.mSize <= T::CAPACITY);
        self.mSize == T::CAPACITY
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear Out Entire Array
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mArray.clear();
        self.mSize = 0;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Access Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn index(&self, index: i32) -> T::TValue {
        assert!(index >= 0 && index < self.mSize);
        self.mArray.index(index as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn index_mut(&mut self, index: i32) -> *mut T::TValue {
        assert!(index >= 0 && index < self.mSize);
        self.mArray.index_mut(index as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access To The Raw Array Pointer
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn raw_array(&self) -> *const T::TValue {
        // this (intentionally) won't compile for anything except value semantics
        // could be done with object semantics, but I would want to assert that all objects are contructed
        self.mArray.raw_array()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access To The Raw Array Pointer
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn raw_array_mut(&mut self) -> *mut T::TValue {
        // this (intentionally) won't compile for anything except value semantics
        // could be done with object semantics, but I would want to assert that all objects are contructed
        self.mArray.raw_array_mut()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn assign(&mut self, val: &vector_base<T>) {
        for i in 0..val.size() {
            self.mArray.construct_with(i as usize, val.mArray.index(i as usize));
        }
        self.mSize = val.mSize;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_back(&mut self) -> *mut T::TValue {
        assert!(self.mSize >= 0 && self.mSize < T::CAPACITY);
        self.mArray.construct(self.mSize as usize);
        self.mSize += 1;
        self.mArray.index_mut((self.mSize - 1) as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add (And Set)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_back_with(&mut self, value: T::TValue) {
        assert!(self.mSize >= 0 && self.mSize < T::CAPACITY);
        self.mArray.construct_with(self.mSize as usize, value);
        self.mSize += 1;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add raw
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_back_raw(&mut self) -> *mut u8 {
        assert!(self.mSize >= 0 && self.mSize < T::CAPACITY);
        self.mSize += 1;
        self.mArray.alloc_raw((self.mSize - 1) as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Remove
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pop_back(&mut self) {
        assert!(self.mSize > 0);
        self.mSize -= 1;
        self.mArray.destruct(self.mSize as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Resizes The Array.  If New Elements Are Needed, It Uses The (value) Param
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn resize_with(&mut self, nSize: i32, value: T::TValue) {
        let mut i = self.mSize - 1;
        while i >= nSize {
            self.mArray.destruct(i as usize);
            self.mSize -= 1;
            i -= 1;
        }
        i = self.mSize;
        while i < nSize {
            self.mArray.construct_with(i as usize, value);
            i += 1;
        }
        self.mSize = nSize;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Resizes The Array.  If New Elements Are Needed, It Uses The (value) Param
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn resize(&mut self, nSize: i32) {
        let mut i = self.mSize - 1;
        while i >= nSize {
            self.mArray.destruct(i as usize);
            self.mSize -= 1;
            i -= 1;
        }
        i = self.mSize;
        while i < nSize {
            self.mArray.construct(i as usize);
            i += 1;
        }
        self.mSize = nSize;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swap the values at two locations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn swap(&mut self, i: i32, j: i32) {
        assert!(i < self.mSize && j < self.mSize);
        self.mArray.swap(i as usize, j as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Erase An Iterator Location... NOTE: THIS DOES NOT PRESERVE ORDER IN THE VECTOR!!
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_swap(&mut self, Index: i32) {
        assert!(Index >= 0 && Index < self.mSize);
        if Index != self.mSize - 1 {
            self.mArray.swap(Index as usize, (self.mSize - 1) as usize);
        }
        self.pop_back();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Binary Search
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_index(&self, value: T::TValue) -> i32 {
        let mut base = 0;
        let mut head = self.mSize - 1;

        loop {
            let searchAt = (base + head) / 2;

            if base == head && searchAt == head {
                break;
            }

            if value < self.mArray.index(searchAt as usize) {
                head = searchAt - 1;
            } else if self.mArray.index(searchAt as usize) < value {
                base = searchAt;
            } else {
                return searchAt;
            }
            if head < base {
                break;
            }
        }

        self.mSize  //not found!
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Heap Sort
    //
    // This sort algorithm has all the advantages of merge sort in terms of guarenteeing
    // O(n log n) worst case, as well as all the advantages of quick sort in that it is
    // "in place" and requires no additional storage.
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn sort(&mut self) {
        // Temporary Data
        //----------------
        let mut HeapSize: i32;  // How Large The Heap Is (Grows In PHASE 1, Shrinks In PHASE 2)
        let mut Pos: i32;  // The Location We Are AT During "re-heapify" Loops
        let mut Compare: i32;  // The Location We Are Comparing AGAINST During "re-heapify" Loops

        // PHASE 1, CONSTRUCT THE HEAP										  O(n log n)
        //===============================================================================
        HeapSize = 1;
        while HeapSize < self.mSize {
            // We Now Have An Element At Heap Size Which Is Not In It's Correct Place
            //------------------------------------------------------------------------
            Pos = HeapSize;
            Compare = Self::parent(Pos);
            while self.mArray.index(Compare as usize) < self.mArray.index(Pos as usize) {
                // Swap The Compare Element With The Pos Element
                //-----------------------------------------------
                self.mArray.swap(Compare as usize, Pos as usize);

                // Move Pos To The Current Compare, And Recalc Compare
                //------------------------------------------------------
                Pos = Compare;
                Compare = Self::parent(Pos);
            }
            HeapSize += 1;
        }

        // PHASE 2, POP OFF THE TOP OF THE HEAP ONE AT A TIME (AND FIX)       O(n log n)
        //===============================================================================
        HeapSize = self.mSize - 1;
        while HeapSize > 0 {
            // Swap The End And Front Of The "Heap" Half Of The Array
            //--------------------------------------------------------
            self.mArray.swap(0, HeapSize as usize);

            // We Now Have A Bogus Element At The Root, So Fix The Heap
            //----------------------------------------------------------
            Pos = 0;
            Compare = self.largest_child(Pos, HeapSize);
            while self.mArray.index(Pos as usize) < self.mArray.index(Compare as usize) {
                // Swap The Compare Element With The Pos Element
                //-----------------------------------------------
                self.mArray.swap(Compare as usize, Pos as usize);

                // Move Pos To The Current Compare, And Recalc Compare
                //------------------------------------------------------
                Pos = Compare;
                Compare = self.largest_child(Pos, HeapSize);
            }
            HeapSize -= 1;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // THIS IS A QUICK VALIDATION OF A SORTED LIST
    ////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub fn sort_validate(&self) {
        for a in 0..(self.mSize - 1) {
            assert!(self.mArray.index(a as usize) < self.mArray.index((a + 1) as usize));
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Node (i)'s Parent Node (The Parent Node Of Zero Is Zero)
    ////////////////////////////////////////////////////////////////////////////////////
    fn parent(i: i32) -> i32 {
        (i - 1) / 2
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Node (i)'s Left Child (The Child Of A Leaf Is The Leaf)
    ////////////////////////////////////////////////////////////////////////////////////
    fn left(i: i32) -> i32 {
        (2 * i) + 1
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Node (i)'s Right Child (The Child Of A Leaf Is The Leaf)
    ////////////////////////////////////////////////////////////////////////////////////
    fn right(i: i32) -> i32 {
        (2 * i) + 2
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Largest Child Of Node (i)
    ////////////////////////////////////////////////////////////////////////////////////
    fn largest_child(&self, i: i32, Size: i32) -> i32 {
        if Self::left(i) < Size {
            if Self::right(i) < Size {
                if self.mArray.index(Self::right(i) as usize) < self.mArray.index(Self::left(i) as usize) {
                    Self::left(i)
                } else {
                    Self::right(i)
                }
            } else {
                Self::left(i)  // Node i only has a left child, so by default it is the biggest
            }
        } else {
            i  // Node i is a leaf, so just return it
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin(&self) -> const_iterator<T> {
        const_iterator {
            mLoc: 0,
            mOwner: self as *const vector_base<T>,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Begin (Starts At Address 0)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_mut(&mut self) -> iterator<T> {
        iterator {
            mLoc: 0,
            mOwner: self as *mut vector_base<T>,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator End (Set To Address mSize)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end(&self) -> const_iterator<T> {
        const_iterator {
            mLoc: self.mSize,
            mOwner: self as *const vector_base<T>,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator End (Set To Address mSize)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end_mut(&mut self) -> iterator<T> {
        iterator {
            mLoc: self.mSize,
            mOwner: self as *mut vector_base<T>,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Find (If Fails To Find, Returns iterator end()
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn find(&mut self, value: T::TValue) -> iterator<T> {
        let index = self.find_index(value);  // Call Find By Index
        if index < self.mSize {
            iterator {
                mLoc: index,  // Found It, Return An Iterator To Index
                mOwner: self as *mut vector_base<T>,
            }
        } else {
            self.end_mut()  // Return "end" Iterator If Not Found
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Find (If Fails To Find, Returns iterator end()
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_const(&self, value: T::TValue) -> const_iterator<T> {
        let index = self.find_index(value);  // Call Find By Index
        if index < self.mSize {
            const_iterator {
                mLoc: index,  // Found It, Return An Iterator To Index
                mOwner: self as *const vector_base<T>,
            }
        } else {
            self.end()  // Return "end" Iterator If Not Found
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Erase An Iterator Location... NOTE: THIS DOES NOT PRESERVE ORDER IN THE VECTOR!!
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_swap_iter(&mut self, it: &iterator<T>) -> iterator<T> {
        assert!(it.mLoc >= 0 && it.mLoc < unsafe { (*it.mOwner).mSize });
        if it.mLoc != self.mSize - 1 {
            self.mArray.swap(it.mLoc as usize, (self.mSize - 1) as usize);
        }
        self.pop_back();
        iterator {
            mLoc: it.mLoc,
            mOwner: self as *mut vector_base<T>,
        }
    }

    pub fn verify_alloc<CAST_TO>(&self, p: *mut CAST_TO) -> *mut CAST_TO {
        self.mArray.verify_alloc(p)
    }
}

////////////////////////////////////////////////////////////////////////////////////
// Iterator
////////////////////////////////////////////////////////////////////////////////////
pub struct iterator<T: TStorageTraits> {
    mLoc: i32,
    mOwner: *mut vector_base<T>,
}

impl<T: TStorageTraits> iterator<T> {
    // Constructors
    //--------------
    pub fn new() -> Self {
        Self {
            mOwner: std::ptr::null_mut(),
            mLoc: 0,
        }
    }

    pub fn from_parts(p: *mut vector_base<T>, t: i32) -> Self {
        Self { mOwner: p, mLoc: t }
    }

    // Assignment Operator
    //---------------------
    pub fn assign(&mut self, t: &iterator<T>) {
        self.mOwner = t.mOwner;
        self.mLoc = t.mLoc;
    }

    // Equality Operators
    //--------------------
    pub fn ne(&self, t: &iterator<T>) -> bool {
        self.mLoc != t.mLoc || self.mOwner != t.mOwner
    }

    pub fn eq(&self, t: &iterator<T>) -> bool {
        self.mLoc == t.mLoc && self.mOwner == t.mOwner
    }

    // DeReference Operator
    //----------------------
    pub fn deref(&self) -> T::TValue {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        unsafe { (*self.mOwner).mArray.index(self.mLoc as usize) }
    }

    // DeReference Operator
    //----------------------
    pub fn value(&self) -> T::TValue {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        unsafe { (*self.mOwner).mArray.index(self.mLoc as usize) }
    }

    // DeReference Operator (pointer access)
    //----------------------
    pub fn arrow(&self) -> *mut T::TValue {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        unsafe { &mut (*self.mOwner).mArray.index_mut(self.mLoc as usize) as *mut _ }
    }

    // Inc Operator (postfix)
    //--------------
    pub fn post_inc(&mut self) -> iterator<T> {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        let old = iterator {
            mLoc: self.mLoc,
            mOwner: self.mOwner,
        };
        self.mLoc += 1;
        old
    }

    // Inc Operator (prefix)
    //--------------
    pub fn pre_inc(&mut self) -> iterator<T> {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        self.mLoc += 1;
        iterator {
            mLoc: self.mLoc,
            mOwner: self.mOwner,
        }
    }
}

impl<T: TStorageTraits> Clone for iterator<T> {
    fn clone(&self) -> Self {
        Self {
            mLoc: self.mLoc,
            mOwner: self.mOwner,
        }
    }
}

impl<T: TStorageTraits> Copy for iterator<T> {}

////////////////////////////////////////////////////////////////////////////////////
// Constant Iterator
////////////////////////////////////////////////////////////////////////////////////
pub struct const_iterator<T: TStorageTraits> {
    mLoc: i32,
    mOwner: *const vector_base<T>,
}

impl<T: TStorageTraits> const_iterator<T> {
    // Constructors
    //--------------
    pub fn new() -> Self {
        Self {
            mOwner: std::ptr::null(),
            mLoc: 0,
        }
    }

    pub fn from_parts(p: *const vector_base<T>, t: i32) -> Self {
        Self { mOwner: p, mLoc: t }
    }

    pub fn from_iterator(t: &iterator<T>) -> Self {
        Self {
            mOwner: t.mOwner as *const vector_base<T>,
            mLoc: t.mLoc,
        }
    }

    // Assignment Operator
    //---------------------
    pub fn assign(&mut self, t: &const_iterator<T>) {
        self.mOwner = t.mOwner;
        self.mLoc = t.mLoc;
    }

    pub fn assign_from_iter(&mut self, t: &iterator<T>) {
        self.mOwner = t.mOwner as *const vector_base<T>;
        self.mLoc = t.mLoc;
    }

    // Equality Operators
    //--------------------
    pub fn ne_iter(&self, t: &iterator<T>) -> bool {
        self.mLoc != t.mLoc || self.mOwner != (t.mOwner as *const vector_base<T>)
    }

    pub fn eq_iter(&self, t: &iterator<T>) -> bool {
        self.mLoc == t.mLoc && self.mOwner == (t.mOwner as *const vector_base<T>)
    }

    pub fn ne(&self, t: &const_iterator<T>) -> bool {
        self.mLoc != t.mLoc || self.mOwner != t.mOwner
    }

    pub fn eq(&self, t: &const_iterator<T>) -> bool {
        self.mLoc == t.mLoc && self.mOwner == t.mOwner
    }

    // DeReference Operator
    //----------------------
    pub fn deref(&self) -> T::TValue {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        unsafe { (*self.mOwner).mArray.index(self.mLoc as usize) }
    }

    // DeReference Operator
    //----------------------
    pub fn value(&self) -> T::TValue {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        unsafe { (*self.mOwner).mArray.index(self.mLoc as usize) }
    }

    // DeReference Operator (pointer access)
    //----------------------
    pub fn arrow(&self) -> *const T::TValue {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        unsafe { &(*self.mOwner).mArray.index(self.mLoc as usize) as *const _ }
    }

    // Inc Operator (postfix)
    //--------------
    pub fn post_inc(&mut self) -> const_iterator<T> {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        let old = const_iterator {
            mLoc: self.mLoc,
            mOwner: self.mOwner,
        };
        self.mLoc += 1;
        old
    }

    // Inc Operator (prefix)
    //--------------
    pub fn pre_inc(&mut self) -> const_iterator<T> {
        assert!(self.mLoc >= 0 && self.mLoc < unsafe { (*self.mOwner).mSize });
        self.mLoc += 1;
        const_iterator {
            mLoc: self.mLoc,
            mOwner: self.mOwner,
        }
    }
}

impl<T: TStorageTraits> Clone for const_iterator<T> {
    fn clone(&self) -> Self {
        Self {
            mLoc: self.mLoc,
            mOwner: self.mOwner,
        }
    }
}

impl<T: TStorageTraits> Copy for const_iterator<T> {}

pub struct vector_vs<T, const ARG_CAPACITY: i32> {
    base: vector_base<storage::value_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: i32> vector_vs<T, ARG_CAPACITY> {
    pub const CAPACITY: i32 = ARG_CAPACITY;

    pub fn new() -> Self {
        Self {
            base: vector_base::new(),
        }
    }
}

pub struct vector_os<T, const ARG_CAPACITY: i32> {
    base: vector_base<storage::object_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: i32> vector_os<T, ARG_CAPACITY> {
    pub const CAPACITY: i32 = ARG_CAPACITY;

    pub fn new() -> Self {
        Self {
            base: vector_base::new(),
        }
    }
}

pub struct vector_is<T, const ARG_CAPACITY: i32, const ARG_MAX_CLASS_SIZE: i32> {
    base: vector_base<storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

impl<T, const ARG_CAPACITY: i32, const ARG_MAX_CLASS_SIZE: i32> vector_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    pub const CAPACITY: i32 = ARG_CAPACITY;
    pub const MAX_CLASS_SIZE: i32 = ARG_MAX_CLASS_SIZE;

    pub fn new() -> Self {
        Self {
            base: vector_base::new(),
        }
    }
}
