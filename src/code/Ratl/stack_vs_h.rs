// ////////////////////////////////////////////////////////////////////////////////////////
// // RAVEN STANDARD TEMPLATE LIBRARY
// //  (c) 2002 Activision
// //
// //
// // Stack
// // -----
// // This is a very simple wrapper around a stack object.
// //
// // First In, Last Out
// //
// //
// //
// // NOTES:
// //
// //
// //
// ////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

use core::ffi::c_int;

// Port note: Storage traits and array_base are defined in ratl_common.h.
// This file defines template specializations that inherit from stack_base parameterized
// by storage trait types. The actual implementation is in the base class.

// Forward declarations / Stubs for unported dependencies
// These represent storage traits and array_base from ratl_common.h

pub struct TRatlNew;

/// Stub trait for storage traits - represents T::TStorageTraits concept from C++
pub trait StorageTraits: Sized {
    type TValue: Clone + Copy;
    const CAPACITY: usize;
}

/// Stub for array_base<T> from ratl_common.h
pub struct array_base<T: StorageTraits> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: StorageTraits> array_base<T> {
    pub fn construct(&mut self, _index: c_int) {
        // Stub - actual implementation in C++ calls storage trait
    }

    pub fn construct_with_value(&mut self, _index: c_int, _value: &T::TValue) {
        // Stub - actual implementation in C++ calls storage trait
    }

    pub fn destruct(&mut self, _index: c_int) {
        // Stub - actual implementation in C++ calls storage trait
    }

    pub fn clear(&mut self) {
        // Stub - actual implementation in C++
    }

    pub fn alloc_raw(&mut self, _index: c_int) -> *mut TRatlNew {
        // Stub - actual implementation in C++
        std::ptr::null_mut()
    }

    pub fn verify_alloc<CAST_TO>(&self, _p: *const CAST_TO) -> *const CAST_TO {
        // Stub - actual implementation in C++
        _p
    }
}

/// Stub for ratl_base from ratl_common.h
pub struct ratl_base;

////////////////////////////////////////////////////////////////////////////////////////
// The stack Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct stack_base<T: StorageTraits> {
    // typedef typename T TStorageTraits;
    // typedef typename T::TValue TTValue;

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Capacity Enum
    // ////////////////////////////////////////////////////////////////////////////////////
    // enum
    // {
    //     CAPACITY        = T::CAPACITY
    // };

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Data
    // ////////////////////////////////////////////////////////////////////////////////////
    mData: array_base<T>, // The Memory
    mSize: c_int,
}

impl<T: StorageTraits> stack_base<T> {
    pub const fn CAPACITY() -> usize {
        T::CAPACITY
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Constructor
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        stack_base {
            mSize: 0,
            mData: array_base {
                _phantom: std::marker::PhantomData,
            },
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Get The Size (The Difference Between The Push And Pop "Pointers")
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> c_int {
        self.mSize
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Check To See If The Size Is Zero
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        self.mSize == 0
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Check To See If The Size Is Full
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        self.mSize >= T::CAPACITY as c_int
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Empty Out The Entire stack
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mSize = 0;
        self.mData.clear();
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Add A Value, returns a reference to the value in place
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn push(&mut self) -> &mut T::TValue {
        assert!(!self.full());
        self.mData.construct(self.mSize);
        self.mSize += 1;
        // Note: This is a stub - actual implementation requires proper indexing into array_base
        unimplemented!("array_base indexing not yet implemented")
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Add A Value to the stack
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_value(&mut self, v: &T::TValue) {
        assert!(!self.full());
        self.mData.construct_with_value(self.mSize, v);
        self.mSize += 1;
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Add A Value to the stack, returning a void * to the memory
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_raw(&mut self) -> *mut TRatlNew {
        assert!(!self.full());
        self.mSize += 1;
        self.mData.alloc_raw(self.mSize - 1)
    }

    // ////////////////////////////////////////////////////////////////////////////////////
    // // Remove A Value From The stack
    // ////////////////////////////////////////////////////////////////////////////////////
    pub fn pop(&mut self) {
        assert!(!self.empty());
        self.mSize -= 1;
        self.mData.destruct(self.mSize);
    }

    pub fn top(&self) -> &T::TValue {
        assert!(!self.empty());
        // Note: This is a stub - actual implementation requires proper indexing into array_base
        unimplemented!("array_base indexing not yet implemented")
    }

    pub fn top_mut(&mut self) -> &mut T::TValue {
        assert!(!self.empty());
        // Note: This is a stub - actual implementation requires proper indexing into array_base
        unimplemented!("array_base indexing not yet implemented")
    }

    pub fn top_const(&self) -> &T::TValue {
        assert!(!self.empty());
        // Note: This is a stub - actual implementation requires proper indexing into array_base
        unimplemented!("array_base indexing not yet implemented")
    }

    pub fn verify_alloc<CAST_TO>(&self, p: *const CAST_TO) -> *const CAST_TO {
        self.mData.verify_alloc(p)
    }
}

// template<class T, int ARG_CAPACITY>
// class stack_vs : public stack_base<storage::value_semantics<T,ARG_CAPACITY> >
pub struct stack_vs<T, const ARG_CAPACITY: usize> {
    // typedef typename storage::value_semantics<T,ARG_CAPACITY> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    // enum
    // {
    //     CAPACITY        = ARG_CAPACITY
    // };

    _phantom: std::marker::PhantomData<T>,
}

impl<T, const ARG_CAPACITY: usize> stack_vs<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    // stack_vs() {}
    pub const fn new() -> Self {
        stack_vs {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, const ARG_CAPACITY: usize> Default for stack_vs<T, ARG_CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// template<class T, int ARG_CAPACITY>
// class stack_os : public stack_base<storage::object_semantics<T,ARG_CAPACITY> >
pub struct stack_os<T, const ARG_CAPACITY: usize> {
    // typedef typename storage::object_semantics<T,ARG_CAPACITY> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    // enum
    // {
    //     CAPACITY        = ARG_CAPACITY
    // };

    _phantom: std::marker::PhantomData<T>,
}

impl<T, const ARG_CAPACITY: usize> stack_os<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    // stack_os() {}
    pub const fn new() -> Self {
        stack_os {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, const ARG_CAPACITY: usize> Default for stack_os<T, ARG_CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// template<class T, int ARG_CAPACITY, int ARG_MAX_CLASS_SIZE>
// class stack_is : public stack_base<storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> >
pub struct stack_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    // typedef typename storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    // enum
    // {
    //     CAPACITY        = ARG_CAPACITY,
    //     MAX_CLASS_SIZE  = ARG_MAX_CLASS_SIZE
    // };

    _phantom: std::marker::PhantomData<T>,
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> stack_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    pub const CAPACITY: usize = ARG_CAPACITY;
    pub const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;

    // stack_is() {}
    pub const fn new() -> Self {
        stack_is {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> Default
    for stack_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}
