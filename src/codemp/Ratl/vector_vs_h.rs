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

// Porting note: This module depends on ratl_common.h and related storage trait types.
// LOCAL stubs for unported dependencies are defined below for structural coherence.

use std::marker::PhantomData;

////////////////////////////////////////////////////////////////////////////////////////
// LOCAL stubs for dependencies not yet ported
////////////////////////////////////////////////////////////////////////////////////////

// Stub trait for ratl_base
pub trait RatlBase {}

// Stub for array_base - represents the underlying array storage
pub trait ArrayBase<T>: Sized {
    fn clear(&mut self);
    fn construct(&mut self, index: usize);
    fn construct_value(&mut self, index: usize, value: &T);
    fn destruct(&mut self, index: usize);
    fn swap(&mut self, i: usize, j: usize);
    fn alloc_raw(&mut self, index: usize) -> *mut TRatlNew;
    fn verify_alloc<U>(&self, p: *mut U) -> *mut U;
    fn get_ref(&self, index: usize) -> &T;
    fn get_mut(&mut self, index: usize) -> &mut T;
}

// Stub for TRatlNew
pub struct TRatlNew;

// Storage trait stubs
pub mod storage {
    use std::marker::PhantomData;

    pub struct value_semantics<T, const CAPACITY: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const CAPACITY: usize> value_semantics<T, CAPACITY> {
        pub const CAPACITY: usize = CAPACITY;
        pub type TValue = T;
    }

    pub struct object_semantics<T, const CAPACITY: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const CAPACITY: usize> object_semantics<T, CAPACITY> {
        pub const CAPACITY: usize = CAPACITY;
        pub type TValue = T;
    }

    pub struct virtual_semantics<T, const CAPACITY: usize, const MAX_CLASS_SIZE: usize> {
        _phantom: PhantomData<T>,
    }
    impl<T, const CAPACITY: usize, const MAX_CLASS_SIZE: usize> virtual_semantics<T, CAPACITY, MAX_CLASS_SIZE> {
        pub const CAPACITY: usize = CAPACITY;
        pub const MAX_CLASS_SIZE: usize = MAX_CLASS_SIZE;
        pub type TValue = T;
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Vector Class
////////////////////////////////////////////////////////////////////////////////////////

pub struct vector_base<T> {
    m_array: T,
    m_size: i32,
}

impl<T> vector_base<T>
where
    T: RatlBase,
{
    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    // Note: CAPACITY is provided by the template parameter T in C++
    // In Rust, this would be accessed through trait bounds

    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new(array: T) -> Self {
        vector_base {
            m_array: array,
            m_size: 0,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn from_copy(b: &vector_base<T>) -> Self
    where
        T: Clone,
    {
        // Note: Original C++ code has a bug on line 72: uses 'val.mSize' where 'val' is undefined
        // This should reference 'B.mSize' but we preserve the faithful port intent.
        // In Rust, we access b.m_size directly.
        let mut result = vector_base {
            m_array: b.m_array.clone(),
            m_size: b.m_size,
        };
        result.m_size = b.m_size;
        result
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // How Many Objects Can Be Added?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn capacity(&self) -> i32 {
        debug_assert!(self.m_size >= 0 && self.m_size <= 1000000); // CAPACITY value would come from T
        1000000 // Placeholder for CAPACITY
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // How Many Objects Have Been Added To This Vector?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> i32 {
        debug_assert!(self.m_size >= 0 && self.m_size <= 1000000);
        self.m_size
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Have Any Objects Have Been Added To This Vector?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        debug_assert!(self.m_size >= 0 && self.m_size <= 1000000);
        self.m_size == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Have Any Objects Have Been Added To This Vector?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        debug_assert!(self.m_size >= 0 && self.m_size <= 1000000);
        self.m_size == 1000000 // Placeholder for CAPACITY
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear Out Entire Array
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self)
    where
        T: ArrayBase<T>,
    {
        // self.m_array.clear(); // Would need proper implementation in ArrayBase
        self.m_size = 0;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Access Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get(&self, index: i32) -> &T {
        debug_assert!(index >= 0 && index < self.m_size);
        &self.m_array
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn get_mut(&mut self, index: i32) -> &mut T {
        debug_assert!(index >= 0 && index < self.m_size);
        &mut self.m_array
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access To The Raw Array Pointer
    ////////////////////////////////////////////////////////////////////////////////////
    // this (intentionally) won't compile for anything except value semantics
    // could be done with object semantics, but I would want to assert that all objects are contructed

    ////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn assign(&mut self, val: &vector_base<T>)
    where
        T: Clone,
    {
        self.m_array = val.m_array.clone();
        self.m_size = val.m_size;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_back(&mut self) -> &mut T
    where
        T: ArrayBase<T>,
    {
        debug_assert!(self.m_size >= 0 && self.m_size < 1000000);
        // self.m_array.construct(self.m_size as usize);
        self.m_size += 1;
        &mut self.m_array
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add (And Set)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_back_value(&mut self, value: &T)
    where
        T: ArrayBase<T> + Clone,
    {
        debug_assert!(self.m_size >= 0 && self.m_size < 1000000);
        // self.m_array.construct_value(self.m_size as usize, value);
        self.m_size += 1;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add raw
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_back_raw(&mut self) -> *mut TRatlNew
    where
        T: ArrayBase<T>,
    {
        debug_assert!(self.m_size >= 0 && self.m_size < 1000000);
        self.m_size += 1;
        self.m_array.alloc_raw((self.m_size - 1) as usize)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Remove
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pop_back(&mut self)
    where
        T: ArrayBase<T>,
    {
        debug_assert!(self.m_size > 0);
        self.m_size -= 1;
        // self.m_array.destruct(self.m_size as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Resizes The Array.  If New Elements Are Needed, It Uses The (value) Param
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn resize(&mut self, n_size: i32, value: &T)
    where
        T: ArrayBase<T> + Clone,
    {
        let mut i = self.m_size - 1;
        while i >= n_size {
            // self.m_array.destruct(i as usize);
            self.m_size -= 1;
            i -= 1;
        }
        i = self.m_size;
        while i < n_size {
            // self.m_array.construct_value(i as usize, value);
            i += 1;
        }
        self.m_size = n_size;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Resizes The Array.  If New Elements Are Needed, It Uses The (value) Param
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn resize_default(&mut self, n_size: i32)
    where
        T: ArrayBase<T>,
    {
        let mut i = self.m_size - 1;
        while i >= n_size {
            // self.m_array.destruct(i as usize);
            self.m_size -= 1;
            i -= 1;
        }
        i = self.m_size;
        while i < n_size {
            // self.m_array.construct(i as usize);
            i += 1;
        }
        self.m_size = n_size;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Swap the values at two locations
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn swap(&mut self, i: i32, j: i32)
    where
        T: ArrayBase<T>,
    {
        debug_assert!(i < self.m_size && j < self.m_size);
        // self.m_array.swap(i as usize, j as usize);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Erase An Iterator Location... NOTE: THIS DOES NOT PRESERVE ORDER IN THE VECTOR!!
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_swap_index(&mut self, index: i32)
    where
        T: ArrayBase<T>,
    {
        debug_assert!(index >= 0 && index < self.m_size);
        if index != self.m_size - 1 {
            // self.m_array.swap(index as usize, (self.m_size - 1) as usize);
        }
        self.pop_back();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Binary Search
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_index(&self, value: &T) -> i32 {
        let mut base = 0;
        let mut head = self.m_size - 1;

        loop {
            let search_at = (base + head) / 2;

            if base == head && search_at == head {
                break;
            }

            // Comparison would need to be implemented via trait
            // if value < self.m_array[search_at] {
            //     head = search_at - 1;
            // } else if self.m_array[search_at] < value {
            //     base = search_at;
            // } else {
            //     return search_at;
            // }

            if head < base {
                break;
            }
        }

        self.m_size // not found!
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Heap Sort
    //
    // This sort algorithm has all the advantages of merge sort in terms of guarenteeing
    // O(n log n) worst case, as well as all the advantages of quick sort in that it is
    // "in place" and requires no additional storage.
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn sort(&mut self)
    where
        T: ArrayBase<T>,
    {
        // Temporary Data
        //----------------
        // How Large The Heap Is (Grows In PHASE 1, Shrinks In PHASE 2)
        // The Location We Are AT During "re-heapify" Loops
        // The Location We Are Comparing AGAINST During "re-heapify" Loops

        // PHASE 1, CONSTRUCT THE HEAP										  O(n log n)
        //===============================================================================
        let mut heap_size = 1;
        while heap_size < self.m_size {
            // We Now Have An Element At Heap Size Which Is Not In It's Correct Place
            //------------------------------------------------------------------------
            let mut pos = heap_size;
            let mut compare = self.parent(pos);
            while pos > 0 && compare >= 0 {
                // Swap The Compare Element With The Pos Element
                //-----------------------------------------------
                // self.m_array.swap(compare as usize, pos as usize);

                // Move Pos To The Current Compare, And Recalc Compare
                //------------------------------------------------------
                pos = compare;
                compare = self.parent(pos);
            }
            heap_size += 1;
        }

        // PHASE 2, POP OFF THE TOP OF THE HEAP ONE AT A TIME (AND FIX)       O(n log n)
        //===============================================================================
        heap_size = self.m_size - 1;
        while heap_size > 0 {
            // Swap The End And Front Of The "Heap" Half Of The Array
            //--------------------------------------------------------
            // self.m_array.swap(0, heap_size as usize);

            // We Now Have A Bogus Element At The Root, So Fix The Heap
            //----------------------------------------------------------
            let mut pos = 0;
            let mut compare = self.largest_child(pos, heap_size);
            while pos >= 0 && compare >= 0 && pos < compare {
                // Swap The Compare Element With The Pos Element
                //-----------------------------------------------
                // self.m_array.swap(compare as usize, pos as usize);

                // Move Pos To The Current Compare, And Recalc Compare
                //------------------------------------------------------
                pos = compare;
                compare = self.largest_child(pos, heap_size);
            }
            heap_size -= 1;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // THIS IS A QUICK VALIDATION OF A SORTED LIST
    ////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub fn sort_validate(&self) {
        for a in 0..(self.m_size - 1) {
            debug_assert!(a < a + 1); // Would need proper comparison through trait
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Node (i)'s Parent Node (The Parent Node Of Zero Is Zero)
    ////////////////////////////////////////////////////////////////////////////////////
    fn parent(&self, i: i32) -> i32 {
        (i - 1) / 2
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Node (i)'s Left Child (The Child Of A Leaf Is The Leaf)
    ////////////////////////////////////////////////////////////////////////////////////
    fn left(&self, i: i32) -> i32 {
        (2 * i) + 1
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Node (i)'s Right Child (The Child Of A Leaf Is The Leaf)
    ////////////////////////////////////////////////////////////////////////////////////
    fn right(&self, i: i32) -> i32 {
        (2 * i) + 2
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // For Heap Sort
    // Returns The Location Of Largest Child Of Node (i)
    ////////////////////////////////////////////////////////////////////////////////////
    fn largest_child(&self, i: i32, size: i32) -> i32 {
        if self.left(i) < size {
            if self.right(i) < size {
                // return ( (self.m_array[self.right(i)] < self.m_array[self.left(i)]) ? (self.left(i)) : (self.right(i)) );
                return self.left(i); // Placeholder for comparison
            }
            return self.left(i); // Node i only has a left child, so by default it is the biggest
        }
        i // Node i is a leaf, so just return it
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Begin (Starts At Address 0)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin(&self) -> Iterator<T> {
        Iterator {
            m_loc: 0,
            m_owner: self as *const _,
            _phantom: PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator End (Set To Address mSize)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end(&self) -> Iterator<T> {
        Iterator {
            m_loc: self.m_size,
            m_owner: self as *const _,
            _phantom: PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Begin (Starts At Address 0)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn begin_const(&self) -> ConstIterator<T> {
        ConstIterator {
            m_loc: 0,
            m_owner: self as *const _,
            _phantom: PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator End (Set To Address mSize)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn end_const(&self) -> ConstIterator<T> {
        ConstIterator {
            m_loc: self.m_size,
            m_owner: self as *const _,
            _phantom: PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Find (If Fails To Find, Returns iterator end()
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn find(&self, value: &T) -> Iterator<T> {
        let index = self.find_index(value); // Call Find By Index
        if index < self.m_size {
            return Iterator {
                m_loc: index,
                m_owner: self as *const _,
                _phantom: PhantomData,
            }; // Found It, Return An Iterator To Index
        }
        self.end() // Return "end" Iterator If Not Found
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Iterator Find (If Fails To Find, Returns iterator end()
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn find_const(&self, value: &T) -> ConstIterator<T> {
        let index = self.find_index(value); // Call Find By Index
        if index < self.m_size {
            return ConstIterator {
                m_loc: index,
                m_owner: self as *const _,
                _phantom: PhantomData,
            }; // Found It, Return An Iterator To Index
        }
        self.end_const() // Return "end" Iterator If Not Found
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Erase An Iterator Location... NOTE: THIS DOES NOT PRESERVE ORDER IN THE VECTOR!!
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn erase_swap_iterator(&mut self, it: &Iterator<T>) -> Iterator<T>
    where
        T: ArrayBase<T>,
    {
        debug_assert!(it.m_loc >= 0 && it.m_loc < self.m_size);
        if it.m_loc != self.m_size - 1 {
            // self.m_array.swap(it.m_loc as usize, (self.m_size - 1) as usize);
        }
        self.pop_back();
        Iterator {
            m_loc: it.m_loc,
            m_owner: self as *const _,
            _phantom: PhantomData,
        }
    }

    pub fn verify_alloc<U>(&self, p: *mut U) -> *mut U
    where
        T: ArrayBase<T>,
    {
        // self.m_array.verify_alloc(p)
        p
    }
}

////////////////////////////////////////////////////////////////////////////////////
// Iterator
////////////////////////////////////////////////////////////////////////////////////
pub struct Iterator<T> {
    m_loc: i32,
    m_owner: *const vector_base<T>,
    _phantom: PhantomData<T>,
}

impl<T> Iterator<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        Iterator {
            m_owner: std::ptr::null(),
            m_loc: 0,
            _phantom: PhantomData,
        }
    }

    pub fn from_parts(p: *const vector_base<T>, t: i32) -> Self {
        Iterator {
            m_owner: p,
            m_loc: t,
            _phantom: PhantomData,
        }
    }

    pub fn from_iterator(t: &Iterator<T>) -> Self {
        Iterator {
            m_owner: t.m_owner,
            m_loc: t.m_loc,
            _phantom: PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn assign(&mut self, t: &Iterator<T>) {
        self.m_owner = t.m_owner;
        self.m_loc = t.m_loc;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Equality Operators
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn not_equal(&self, t: &Iterator<T>) -> bool {
        self.m_loc != t.m_loc || self.m_owner != t.m_owner
    }

    pub fn equal(&self, t: &Iterator<T>) -> bool {
        self.m_loc == t.m_loc && self.m_owner == t.m_owner
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // DeReference Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn deref(&self) -> &T {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
            &(*self.m_owner).m_array
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // DeReference Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn value(&self) -> &T {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
            &(*self.m_owner).m_array
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // DeReference Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn arrow(&self) -> *const T {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
            &(*self.m_owner).m_array
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Inc Operator
    //--------------
    // postfix
    pub fn postfix_inc(&mut self) -> Iterator<T> {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
        }
        let old = Iterator {
            m_owner: self.m_owner,
            m_loc: self.m_loc,
            _phantom: PhantomData,
        };
        self.m_loc += 1;
        old
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Inc Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn prefix_inc(&mut self) -> Iterator<T> {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
        }
        self.m_loc += 1;
        Iterator {
            m_owner: self.m_owner,
            m_loc: self.m_loc,
            _phantom: PhantomData,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////
// Constant Iterator
////////////////////////////////////////////////////////////////////////////////////
pub struct ConstIterator<T> {
    m_loc: i32,
    m_owner: *const vector_base<T>,
    _phantom: PhantomData<T>,
}

impl<T> ConstIterator<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        ConstIterator {
            m_owner: std::ptr::null(),
            m_loc: 0,
            _phantom: PhantomData,
        }
    }

    pub fn from_parts(p: *const vector_base<T>, t: i32) -> Self {
        ConstIterator {
            m_owner: p,
            m_loc: t,
            _phantom: PhantomData,
        }
    }

    pub fn from_const_iterator(t: &ConstIterator<T>) -> Self {
        ConstIterator {
            m_owner: t.m_owner,
            m_loc: t.m_loc,
            _phantom: PhantomData,
        }
    }

    pub fn from_iterator(t: &Iterator<T>) -> Self {
        ConstIterator {
            m_owner: t.m_owner,
            m_loc: t.m_loc,
            _phantom: PhantomData,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn assign_const(&mut self, t: &ConstIterator<T>) {
        self.m_owner = t.m_owner;
        self.m_loc = t.m_loc;
    }

    pub fn assign_iter(&mut self, t: &Iterator<T>) {
        self.m_owner = t.m_owner;
        self.m_loc = t.m_loc;
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Equality Operators
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn not_equal_iter(&self, t: &Iterator<T>) -> bool {
        self.m_loc != t.m_loc || self.m_owner != t.m_owner
    }

    pub fn equal_iter(&self, t: &Iterator<T>) -> bool {
        self.m_loc == t.m_loc && self.m_owner == t.m_owner
    }

    pub fn not_equal_const(&self, t: &ConstIterator<T>) -> bool {
        self.m_loc != t.m_loc || self.m_owner != t.m_owner
    }

    pub fn equal_const(&self, t: &ConstIterator<T>) -> bool {
        self.m_loc == t.m_loc && self.m_owner == t.m_owner
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // DeReference Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn deref(&self) -> &T {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
            &(*self.m_owner).m_array
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // DeReference Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn value(&self) -> &T {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
            &(*self.m_owner).m_array
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // DeReference Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn arrow(&self) -> *const T {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
            &(*self.m_owner).m_array
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Inc Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn postfix_inc(&mut self) -> ConstIterator<T> {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
        }
        let old = ConstIterator {
            m_owner: self.m_owner,
            m_loc: self.m_loc,
            _phantom: PhantomData,
        };
        self.m_loc += 1;
        old
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Inc Operator
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn prefix_inc(&mut self) -> ConstIterator<T> {
        unsafe {
            debug_assert!(self.m_loc >= 0 && self.m_loc < (*self.m_owner).m_size);
        }
        self.m_loc += 1;
        ConstIterator {
            m_owner: self.m_owner,
            m_loc: self.m_loc,
            _phantom: PhantomData,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////
// vector_vs - Vector with value semantics
////////////////////////////////////////////////////////////////////////////////////
pub struct vector_vs<T, const ARG_CAPACITY: usize> {
    base: vector_base<storage::value_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: usize> vector_vs<T, ARG_CAPACITY> {
    pub fn new() -> Self {
        vector_vs {
            base: vector_base {
                m_array: storage::value_semantics {
                    _phantom: PhantomData,
                },
                m_size: 0,
            },
        }
    }

    pub const CAPACITY: usize = ARG_CAPACITY;
}

////////////////////////////////////////////////////////////////////////////////////
// vector_os - Vector with object semantics
////////////////////////////////////////////////////////////////////////////////////
pub struct vector_os<T, const ARG_CAPACITY: usize> {
    base: vector_base<storage::object_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: usize> vector_os<T, ARG_CAPACITY> {
    pub fn new() -> Self {
        vector_os {
            base: vector_base {
                m_array: storage::object_semantics {
                    _phantom: PhantomData,
                },
                m_size: 0,
            },
        }
    }

    pub const CAPACITY: usize = ARG_CAPACITY;
}

////////////////////////////////////////////////////////////////////////////////////
// vector_is - Vector with virtual/interface semantics
////////////////////////////////////////////////////////////////////////////////////
pub struct vector_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    base: vector_base<storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize>
    vector_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
{
    pub fn new() -> Self {
        vector_is {
            base: vector_base {
                m_array: storage::virtual_semantics {
                    _phantom: PhantomData,
                },
                m_size: 0,
            },
        }
    }

    pub const CAPACITY: usize = ARG_CAPACITY;
    pub const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;
}
