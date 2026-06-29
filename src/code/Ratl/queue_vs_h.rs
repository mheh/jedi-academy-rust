////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Queue Template
// --------------
// The queue is a circular buffer of objects which supports a push at the "front" and a
// pop at the "back".  Therefore it is:
//
// First In, First Out
//
// As the pointers to the push and pop locations are changed it wrapps around the end
// of the array and back to the front.  There are asserts to make sure it never goes
// beyond the capacity of the object.
//
//
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

// Stub for external RATL dependencies - actual implementations would come from ratl_common.h and array_base.h
pub trait ratl_base {}

pub trait TRatlStorageTraits {
    type TValue;
    const CAPACITY: usize;
}

pub struct array_base<T: TRatlStorageTraits> {
    _phantom: core::marker::PhantomData<T>,
}

impl<T: TRatlStorageTraits> array_base<T> {
    pub fn new() -> Self {
        array_base {
            _phantom: core::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {}
    pub fn construct(&mut self, _idx: usize) {}
    pub fn construct_value(&mut self, _idx: usize, _v: &T::TValue) {}
    pub fn destruct(&mut self, _idx: usize) {}
    pub fn alloc_raw(&mut self, _idx: usize) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    pub fn verify_alloc<CAST_TO>(&self, p: *const CAST_TO) -> *const CAST_TO {
        p
    }
    pub fn index_get(&self, _idx: usize) -> &T::TValue {
        unsafe { &*(core::ptr::null() as *const T::TValue) }
    }
    pub fn index_get_mut(&mut self, _idx: usize) -> &mut T::TValue {
        unsafe { &mut *(core::ptr::null_mut() as *mut T::TValue) }
    }
}

pub struct TRatlNew;

////////////////////////////////////////////////////////////////////////////////////////
// The Queue Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct queue_base<T: TRatlStorageTraits> {
    mData: array_base<T>,                      // The Memory
    mPush: i32,                                // Address Of Next Add Location
    mPop: i32,                                 // Address Of Next Remove Location
    mSize: i32,
}

impl<T: TRatlStorageTraits> queue_base<T> {
    type TStorageTraits = T;
    type TTValue = T::TValue;

    const CAPACITY: usize = T::CAPACITY;

    fn push_low(&mut self) -> usize {
        assert!(self.size() < Self::CAPACITY as i32);

        // Add It
        //--------
        self.mPush += 1;
        self.mSize += 1;

        // Update Push Location
        //----------------------
        if self.mPush >= Self::CAPACITY as i32 {
            self.mPush = 0;
            return Self::CAPACITY - 1;
        }
        (self.mPush - 1) as usize
    }


    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        queue_base {
            mData: array_base::new(),
            mPush: 0,
            mPop: 0,
            mSize: 0,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Get The Size (The Difference Between The Push And Pop "Pointers")
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> i32 {
        self.mSize
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If The Size Is Zero
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        self.mSize == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Check To See If The Size Is Full
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        self.mSize >= Self::CAPACITY as i32
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Empty Out The Entire Queue
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mPush = 0;
        self.mPop = 0;
        self.mSize = 0;
        self.mData.clear();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add A Value, returns a reference to the value in place
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push(&mut self) -> &mut T::TValue {
        let idx = self.push_low();
        self.mData.construct(idx);
        self.mData.index_get_mut(idx)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add A Value to the Queue
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_value(&mut self, v: &T::TValue) {
        let idx = self.push_low();
        self.mData.construct_value(idx, v);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add A Value to the Queue, returning a void * to the memory
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn push_raw(&mut self) -> *mut core::ffi::c_void {
        let idx = self.push_low();
        self.mData.alloc_raw(idx)
    }
    ////////////////////////////////////////////////////////////////////////////////////
    // Remove A Value From The Queue
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn pop(&mut self) {
        assert!(self.size() > 0);

        self.mData.destruct(self.mPop as usize);
        // Update Pop Location
        //---------------------
        self.mPop += 1;
        if self.mPop >= Self::CAPACITY as i32 {
            self.mPop = 0;
        }

        self.mSize -= 1;
    }

    pub fn top(&self) -> &T::TValue {
        assert!(self.size() > 0);
        self.mData.index_get(self.mPop as usize)
    }

    pub fn top_mut(&mut self) -> &mut T::TValue {
        assert!(self.size() > 0);
        self.mData.index_get_mut(self.mPop as usize)
    }

    pub fn verify_alloc<CAST_TO>(&self, p: *const CAST_TO) -> *const CAST_TO {
        self.mData.verify_alloc(p)
    }
}

pub mod storage {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////////////////
    // value_semantics Storage Trait
    ////////////////////////////////////////////////////////////////////////////////////////
    pub struct value_semantics<T, const ARG_CAPACITY: usize> {
        _phantom: core::marker::PhantomData<T>,
    }

    impl<T, const ARG_CAPACITY: usize> TRatlStorageTraits for value_semantics<T, ARG_CAPACITY> {
        type TValue = T;
        const CAPACITY: usize = ARG_CAPACITY;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // object_semantics Storage Trait
    ////////////////////////////////////////////////////////////////////////////////////////
    pub struct object_semantics<T, const ARG_CAPACITY: usize> {
        _phantom: core::marker::PhantomData<T>,
    }

    impl<T, const ARG_CAPACITY: usize> TRatlStorageTraits for object_semantics<T, ARG_CAPACITY> {
        type TValue = T;
        const CAPACITY: usize = ARG_CAPACITY;
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // virtual_semantics Storage Trait
    ////////////////////////////////////////////////////////////////////////////////////////
    pub struct virtual_semantics<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
        _phantom: core::marker::PhantomData<T>,
    }

    impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> TRatlStorageTraits for virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
        type TValue = T;
        const CAPACITY: usize = ARG_CAPACITY;
    }
}

pub struct queue_vs<T, const ARG_CAPACITY: usize> {
    base: queue_base<storage::value_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: usize> queue_vs<T, ARG_CAPACITY> {
    type TStorageTraits = storage::value_semantics<T, ARG_CAPACITY>;
    type TTValue = T;

    const CAPACITY: usize = ARG_CAPACITY;

    pub fn new() -> Self {
        queue_vs {
            base: queue_base::new(),
        }
    }
}

pub struct queue_os<T, const ARG_CAPACITY: usize> {
    base: queue_base<storage::object_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: usize> queue_os<T, ARG_CAPACITY> {
    type TStorageTraits = storage::object_semantics<T, ARG_CAPACITY>;
    type TTValue = T;

    const CAPACITY: usize = ARG_CAPACITY;

    pub fn new() -> Self {
        queue_os {
            base: queue_base::new(),
        }
    }
}

pub struct queue_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    base: queue_base<storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> queue_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    type TStorageTraits = storage::virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>;
    type TTValue = T;

    const CAPACITY: usize = ARG_CAPACITY;
    const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;

    pub fn new() -> Self {
        queue_is {
            base: queue_base::new(),
        }
    }
}
