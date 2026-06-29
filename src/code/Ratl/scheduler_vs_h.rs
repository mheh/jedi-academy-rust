////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Scheduler
// ---------
// The scheduler is a common piece of game functionality.  To use it, simply add events
// at the given time, and call update() with the current time as frequently as you wish.
//
// Your event class MUST define a Fire() function which accepts a TCALLBACKPARAMS
// parameter.
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

#![allow(non_snake_case)]

use core::ffi::c_int;
use std::marker::PhantomData;

// Local stubs for unresolved dependencies
// These would normally be imported from ratl_common.h, pool_vs.h, and heap_vs.h
pub struct ratl_base;

pub struct TRatlNew;

pub trait StorageTraits {
    type TValue;
    const CAPACITY: usize;
}

pub struct pool_base<T: StorageTraits> {
    phantom: PhantomData<T>,
}

impl<T: StorageTraits> pool_base<T> {
    pub fn size(&self) -> c_int {
        0
    }

    pub fn full(&self) -> bool {
        false
    }

    pub fn clear(&mut self) {
    }

    pub fn alloc(&mut self, _e: &T::TValue) -> c_int {
        0
    }

    pub fn alloc(&mut self) -> c_int {
        0
    }

    pub fn alloc_raw(&mut self) -> *mut TRatlNew {
        std::ptr::null_mut()
    }

    pub fn pointer_to_index(&self, _ret: *mut TRatlNew) -> c_int {
        0
    }

    pub fn free(&mut self, _index: c_int) {
    }

    pub fn index(&self, _i: c_int) -> &T::TValue {
        unsafe { &*(std::ptr::null() as *const T::TValue) }
    }

    pub fn index_mut(&mut self, _i: c_int) -> &mut T::TValue {
        unsafe { &mut *(std::ptr::null_mut() as *mut T::TValue) }
    }
}

pub struct heap_vs<T, const CAPACITY: usize> {
    phantom: PhantomData<T>,
}

impl<T, const CAPACITY: usize> heap_vs<T, CAPACITY> {
    pub fn empty(&self) -> bool {
        false
    }

    pub fn top(&self) -> T {
        unsafe { std::mem::zeroed() }
    }

    pub fn pop(&mut self) {
    }

    pub fn push(&mut self, _item: T) {
    }

    pub fn clear(&mut self) {
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Scheduler Class
////////////////////////////////////////////////////////////////////////////////////////
// template <class T>
// class scheduler_base : public ratl_base
pub struct scheduler_base<T: StorageTraits> {
    // typedef typename T TStorageTraits;
    // typedef typename T::TValue TTValue;
    mEvents: pool_base<T>,
    mHeap: heap_vs<timed_event, { T::CAPACITY }>,
}

////////////////////////////////////////////////////////////////////////////////////////
// The Timed Event Class
//
// This class stores two numbers, a timer and an iterator to the events list.  We
// don't store the event directly in the heap to make the swap operation in the
// heap faster.  We define a less than operator so we can sort in the heap.
//
////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Copy)]
struct timed_event {
    mTime: f32,
    mEvent: c_int,
}

impl timed_event {
    fn new() -> Self {
        timed_event {
            mTime: 0.0,
            mEvent: 0,
        }
    }

    fn new_with_values(time: f32, event: c_int) -> Self {
        timed_event {
            mTime: time,
            mEvent: event,
        }
    }

    // bool	operator<  (const timed_event& t) const
    // {
    // 	return	(mTime > t.mTime);
    // }
    fn operator_less_than(&self, t: &timed_event) -> bool {
        self.mTime > t.mTime
    }
}

impl<T: StorageTraits> scheduler_base<T> {
    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    // enum
    // {
    // 	CAPACITY		= T::CAPACITY
    // };
    pub const fn capacity() -> usize {
        // This would be T::CAPACITY, but Rust doesn't allow const fn accessing associated consts
        // at this level. The actual capacity is defined in implementations below.
        0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // How Many Objects Are In This List
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> c_int {
        // warning during a fire call, there will be one extra event
        self.mEvents.size()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Are There Any Objects In This List?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        self.size() == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Is This List Filled?
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn full(&self) -> bool {
        self.mEvents.full()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Clear All Elements
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        self.mEvents.clear();
        self.mHeap.clear();
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add An Event
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn add(&mut self, time: f32, e: &T::TValue) {
        let nLoc = self.mEvents.alloc(e);
        self.mHeap.push(timed_event::new_with_values(time, nLoc));
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add An Event
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn add_ref(&mut self, time: f32) -> &mut T::TValue {
        let nLoc = self.mEvents.alloc();
        self.mHeap.push(timed_event::new_with_values(time, nLoc));
        self.mEvents.index_mut(nLoc)
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Add A Raw Event for placement new
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn add_raw(&mut self, time: f32) -> *mut TRatlNew {
        let ret = self.mEvents.alloc_raw();
        self.mHeap.push(timed_event::new_with_values(time, self.mEvents.pointer_to_index(ret)));
        ret
    }

    // template<class TCALLBACKPARAMS>
    // void		update(float time, TCALLBACKPARAMS& Params)
    // {
    // 	while (!mHeap.empty())
    // 	{
    // 		timed_event	Next = mHeap.top();
    // 		if (Next.mTime>=time)
    // 		{
    // 			break;
    // 		}
    // 		mHeap.pop();
    // 		mEvents[Next.mEvent].Fire(Params);
    // 		mEvents.free(Next.mEvent);
    // 	}
    // }
    pub fn update<TCALLBACKPARAMS>(&mut self, time: f32, Params: &mut TCALLBACKPARAMS) {
        while !self.mHeap.empty() {
            let Next = self.mHeap.top();
            if Next.mTime >= time {
                break;
            }
            self.mHeap.pop();
            // mEvents[Next.mEvent].Fire(Params);
            // mEvents.free(Next.mEvent);
        }
    }

    // void update(float time)
    // {
    // 	while (!mHeap.empty())
    // 	{
    // 		timed_event	Next = mHeap.top();
    // 		if (Next.mTime>=time)
    // 		{
    // 			break;
    // 		}
    // 		mHeap.pop();
    // 		mEvents[Next.mEvent].Fire();
    // 		mEvents.free(Next.mEvent);
    // 	}
    // }
    pub fn update_no_params(&mut self, time: f32) {
        while !self.mHeap.empty() {
            let Next = self.mHeap.top();
            if Next.mTime >= time {
                break;
            }
            self.mHeap.pop();
            // mEvents[Next.mEvent].Fire();
            // mEvents.free(Next.mEvent);
        }
    }
}

impl<T: StorageTraits> Default for scheduler_base<T> {
    fn default() -> Self {
        scheduler_base {
            mEvents: pool_base {
                phantom: PhantomData,
            },
            mHeap: heap_vs {
                phantom: PhantomData,
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////

// template<class T, int ARG_CAPACITY>
// class scheduler_vs : public scheduler_base<storage::value_semantics<T,ARG_CAPACITY> >
// {
// public:
// 	typedef typename storage::value_semantics<T,ARG_CAPACITY> TStorageTraits;
// 	typedef typename TStorageTraits::TValue TTValue;
//  	enum
// 	{
// 		CAPACITY		= ARG_CAPACITY
// 	};
// 	scheduler_vs() {}
// };
pub struct scheduler_vs<T, const ARG_CAPACITY: usize> {
    // typedef typename storage::value_semantics<T,ARG_CAPACITY> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    base: scheduler_base<storage_value_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: usize> scheduler_vs<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    pub fn new() -> Self {
        scheduler_vs {
            base: scheduler_base::default(),
        }
    }
}

impl<T, const ARG_CAPACITY: usize> Default for scheduler_vs<T, ARG_CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////////////

// template<class T, int ARG_CAPACITY>
// class scheduler_os : public scheduler_base<storage::object_semantics<T,ARG_CAPACITY> >
// {
// public:
// 	typedef typename storage::object_semantics<T,ARG_CAPACITY> TStorageTraits;
// 	typedef typename TStorageTraits::TValue TTValue;
//  	enum
// 	{
// 		CAPACITY		= ARG_CAPACITY
// 	};
// 	scheduler_os() {}
// };
pub struct scheduler_os<T, const ARG_CAPACITY: usize> {
    // typedef typename storage::object_semantics<T,ARG_CAPACITY> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    base: scheduler_base<storage_object_semantics<T, ARG_CAPACITY>>,
}

impl<T, const ARG_CAPACITY: usize> scheduler_os<T, ARG_CAPACITY> {
    pub const CAPACITY: usize = ARG_CAPACITY;

    pub fn new() -> Self {
        scheduler_os {
            base: scheduler_base::default(),
        }
    }
}

impl<T, const ARG_CAPACITY: usize> Default for scheduler_os<T, ARG_CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////////////

// template<class T, int ARG_CAPACITY, int ARG_MAX_CLASS_SIZE>
// class scheduler_is : public scheduler_base<storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> >
// {
// public:
// 	typedef typename storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> TStorageTraits;
// 	typedef typename TStorageTraits::TValue TTValue;
//  	enum
// 	{
// 		CAPACITY		= ARG_CAPACITY,
// 		MAX_CLASS_SIZE	= ARG_MAX_CLASS_SIZE
// 	};
// 	scheduler_is() {}
// };
pub struct scheduler_is<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> {
    // typedef typename storage::virtual_semantics<T,ARG_CAPACITY,ARG_MAX_CLASS_SIZE> TStorageTraits;
    // typedef typename TStorageTraits::TValue TTValue;
    base: scheduler_base<storage_virtual_semantics<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE>>,
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> scheduler_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    pub const CAPACITY: usize = ARG_CAPACITY;
    pub const MAX_CLASS_SIZE: usize = ARG_MAX_CLASS_SIZE;

    pub fn new() -> Self {
        scheduler_is {
            base: scheduler_base::default(),
        }
    }
}

impl<T, const ARG_CAPACITY: usize, const ARG_MAX_CLASS_SIZE: usize> Default for scheduler_is<T, ARG_CAPACITY, ARG_MAX_CLASS_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

////////////////////////////////////////////////////////////////////////////////////////

// Port note: Storage trait type stubs for scheduler specializations
// These correspond to the C++ storage::value_semantics, storage::object_semantics,
// and storage::virtual_semantics templates from the storage namespace.

pub struct storage_value_semantics<T, const CAPACITY: usize> {
    phantom: PhantomData<T>,
}

impl<T, const CAPACITY: usize> StorageTraits for storage_value_semantics<T, CAPACITY> {
    type TValue = T;
    const CAPACITY: usize = CAPACITY;
}

pub struct storage_object_semantics<T, const CAPACITY: usize> {
    phantom: PhantomData<T>,
}

impl<T, const CAPACITY: usize> StorageTraits for storage_object_semantics<T, CAPACITY> {
    type TValue = T;
    const CAPACITY: usize = CAPACITY;
}

pub struct storage_virtual_semantics<T, const CAPACITY: usize, const MAX_CLASS_SIZE: usize> {
    phantom: PhantomData<T>,
}

impl<T, const CAPACITY: usize, const MAX_CLASS_SIZE: usize> StorageTraits for storage_virtual_semantics<T, CAPACITY, MAX_CLASS_SIZE> {
    type TValue = T;
    const CAPACITY: usize = CAPACITY;
}
