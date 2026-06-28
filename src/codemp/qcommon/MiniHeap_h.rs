//! `MiniHeap.h` — compact C++ mini-heap helper.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

unsafe extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

#[repr(C)]
pub struct CMiniHeap {
    mHeap: *mut i8,
    mCurrentHeap: *mut i8,
    mSize: c_int,
}

impl CMiniHeap {
    // reset the heap back to the start
    pub unsafe fn ResetHeap(&mut self) {
        self.mCurrentHeap = self.mHeap;
    }

    // initialise the heap
    pub unsafe fn CMiniHeap(size: c_int) -> Self {
        let mut heap = Self {
            mHeap: unsafe { malloc(size as usize) as *mut i8 },
            mCurrentHeap: null_mut(),
            mSize: size,
        };
        if !heap.mHeap.is_null() {
            unsafe { heap.ResetHeap() };
        }
        heap
    }

    // give me some space from the heap please
    pub unsafe fn MiniHeapAlloc(&mut self, size: c_int) -> *mut i8 {
        if size
            < self
                .mSize
                .wrapping_sub((self.mCurrentHeap as isize).wrapping_sub(self.mHeap as isize) as c_int)
        {
            let tempAddress: *mut i8 = self.mCurrentHeap;
            self.mCurrentHeap = unsafe { self.mCurrentHeap.add(size as usize) };
            return tempAddress;
        }
        null_mut()
    }
}

impl Drop for CMiniHeap {
    // free up the heap
    fn drop(&mut self) {
        unsafe {
            if !self.mHeap.is_null() {
                free(self.mHeap as *mut c_void);
            }
        }
    }
}

unsafe extern "C" {
    pub static mut G2VertSpaceServer: *mut CMiniHeap;
    pub static mut G2VertSpaceClient: *mut CMiniHeap;
}
