// ////////////////////////////////////////////////////////////////////////////////////////
// // RAVEN STANDARD TEMPLATE LIBRARY
// //  (c) 2002 Activision
// //
// //
// // Array
// // -----
// // This array class is little more than an assert loaded wrapper around a standard
// // array.
// //
// //
// //
// // NOTES:
// //
// //
// //
// ////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

// Port note: array_base and storage trait types are defined in ratl_common.h.
// This file defines template specializations that inherit from array_base parameterized
// by storage trait types. The actual implementation is in the base class.

// template<class T, int ARG_CAPACITY>
// class array_vs : public array_base<storage::value_semantics<T,ARG_CAPACITY> >
pub struct array_vs<T, const ARG_CAPACITY: usize> {
    // typedef typename storage::value_semantics<T,ARG_CAPACITY> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    // Port note: Type aliases reference external types from storage traits.
    // The actual storage behavior is implemented in array_base.

    _phantom: std::marker::PhantomData<T>,
}

// enum { CAPACITY = ARG_CAPACITY };
impl<T, const ARG_CAPACITY: usize> array_vs<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    // array_vs() {}
    pub const fn new() -> Self {
        array_vs {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, const ARG_CAPACITY: usize> Default for array_vs<T, ARG_CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// template<class T, int ARG_CAPACITY>
// class array_os : public array_base<storage::object_semantics<T,ARG_CAPACITY> >
pub struct array_os<T, const ARG_CAPACITY: usize> {
    // typedef typename storage::object_semantics<T,ARG_CAPACITY> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    // Port note: Type aliases reference external types from storage traits.
    // The actual storage behavior is implemented in array_base.

    _phantom: std::marker::PhantomData<T>,
}

// enum { CAPACITY = ARG_CAPACITY };
impl<T, const ARG_CAPACITY: usize> array_os<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    // array_os() {}
    pub const fn new() -> Self {
        array_os {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, const ARG_CAPACITY: usize> Default for array_os<T, ARG_CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// template<class T, int ARG_CAPACITY, int ARG_MAX_CLASS_SIZE>
// class array_is : public array_base<storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> >
pub struct array_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    // typedef typename storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    // Port note: Type aliases reference external types from storage traits.
    // The actual storage behavior is implemented in array_base.

    _phantom: std::marker::PhantomData<T>,
}

// enum { CAPACITY = ARG_CAPACITY, MAX_CLASS_SIZE = ARG_MAX_CLASS_SIZE };
impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> array_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    pub const CAPACITY: usize = ARG_CAPACITY;
    pub const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;

    // array_is() {}
    pub const fn new() -> Self {
        array_is {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> Default
    for array_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>
{
    fn default() -> Self {
        Self::new()
    }
}
