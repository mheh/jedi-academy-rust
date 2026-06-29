use core::ffi::{c_int, c_char};
use std::ptr;

#[repr(C)]
#[allow(non_snake_case)]
pub struct CMiniHeap {
    mHeap: *mut c_char,
    mCurrentHeap: *mut c_char,
    mSize: c_int,
    #[cfg(debug_assertions)]
    mMaxAlloc: c_int,
}

extern "C" {
    fn Z_Malloc(size: c_int, tag: c_int, clearFlag: c_int) -> *mut c_char;
    static TAG_GHOUL2: c_int;
    static qtrue: c_int;
    pub static mut G2VertSpaceServer: *mut CMiniHeap;
}

impl CMiniHeap {
    // reset the heap back to the start
    pub fn ResetHeap(&mut self) {
        #[cfg(debug_assertions)]
        {
            if (self.mCurrentHeap as c_int) - (self.mHeap as c_int) > self.mMaxAlloc {
                self.mMaxAlloc = (self.mCurrentHeap as c_int) - (self.mHeap as c_int);
            }
        }
        self.mCurrentHeap = self.mHeap;
    }

    // initialise the heap
    pub fn new(size: c_int) -> Self {
        unsafe {
            let heap = Z_Malloc(size, TAG_GHOUL2, qtrue);
            let mut instance = CMiniHeap {
                mHeap: heap,
                mCurrentHeap: heap,
                mSize: size,
                #[cfg(debug_assertions)]
                mMaxAlloc: 0,
            };
            if !heap.is_null() {
                instance.ResetHeap();
            }
            instance
        }
    }

    // give me some space from the heap please
    pub fn MiniHeapAlloc(&mut self, size: c_int) -> *mut c_char {
        if size < (self.mSize - ((self.mCurrentHeap as c_int) - (self.mHeap as c_int))) {
            let tempAddress = self.mCurrentHeap;
            self.mCurrentHeap = unsafe { self.mCurrentHeap.offset(size as isize) };
            tempAddress
        } else {
            ptr::null_mut()
        }
    }
}

// free up the heap
impl Drop for CMiniHeap {
    fn drop(&mut self) {
        if !self.mHeap.is_null() {
            // the quake heap will be long gone, no need to free it Z_Free(mHeap);
        }
    }
}
