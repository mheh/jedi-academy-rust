#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_int, c_uint, c_void};
use core::mem::{size_of, MaybeUninit};
use core::ptr::null_mut;
use std::alloc::{alloc, dealloc, handle_alloc_error, Layout};

// C origin: `codemp/qcommon/fixedmap.h`.

/*
   An STL map-like container.  Quickly thrown together to replace STL maps
   in specific instances.  Many gotchas.  Use with caution.
*/

type cmp_t = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;

unsafe extern "C" {
    fn qsort(base: *mut c_void, nmemb: usize, size: usize, compar: cmp_t);
}

#[repr(C)]
pub struct Data<T: Copy, U: Copy + PartialOrd> {
    data: T,
    key: U,
}

#[repr(C)]
pub struct VVFixedMap<T: Copy, U: Copy + PartialOrd> {
    items: *mut Data<T, U>,
    numItems: c_uint,
    maxItems: c_uint,
}

impl<T: Copy, U: Copy + PartialOrd> VVFixedMap<T, U> {
    // Porting deviation: Rust cannot overload the public C++ constructor and private
    // default constructor, so the private `VVFixedMap(void) {}` is named explicitly.
    unsafe fn VVFixedMap_void() -> Self {
        Self {
            items: null_mut(),
            numItems: 0,
            maxItems: 0,
        }
    }

    pub unsafe fn VVFixedMap(maxItems: c_uint) -> Self {
        let layout = Layout::array::<Data<T, U>>(maxItems as usize).unwrap();
        let items = if layout.size() == 0 {
            null_mut()
        } else {
            let items = unsafe { alloc(layout) as *mut Data<T, U> };
            if items.is_null() {
                handle_alloc_error(layout);
            }
            items
        };

        Self {
            items,
            numItems: 0,
            maxItems,
        }
    }

    pub unsafe fn Insert(&mut self, newItem: *const T, key: *const U) -> bool {
        let mut storage: *mut Data<T, U> = null_mut();

        //Check for fullness.
        if self.numItems >= self.maxItems {
            return false;
        }

        //Check for reuse.
        if unsafe { self.FindUnsorted(key, &mut storage) }.is_null() {
            storage = unsafe { self.items.add(self.numItems as usize) };
            self.numItems += 1;
        }

        unsafe {
            (*storage).data = *newItem;
            (*storage).key = *key;
        }

        true
    }

    pub unsafe fn Sort(&mut self) {
        unsafe {
            qsort(
                self.items as *mut c_void,
                self.numItems as usize,
                size_of::<Data<T, U>>(),
                Some(Self::FixedMapSorter),
            );
        }
    }

    //Binary search, items must have been sorted!
    pub unsafe fn Find(&mut self, key: *const U) -> *mut T {
        let mut i: MaybeUninit<c_int> = MaybeUninit::uninit();
        let mut high: c_int;
        let mut low: c_int;

        low = -1;
        high = self.numItems as c_int;
        while high - low > 1 {
            let i_value = (high + low) / 2;
            i.write(i_value);
            if unsafe { *key < (*self.items.add(i_value as usize)).key } {
                high = i_value;
            } else if unsafe { *key > (*self.items.add(i_value as usize)).key } {
                low = i_value;
            } else {
                return unsafe { &mut (*self.items.add(i_value as usize)).data as *mut T };
            }
        }

        let i = unsafe { i.assume_init() };
        if unsafe { (*self.items.offset((i + 1) as isize)).key == *key } {
            unsafe { &mut (*self.items.offset((i + 1) as isize)).data as *mut T }
        } else if unsafe { (*self.items.offset((i - 1) as isize)).key == *key } {
            unsafe { &mut (*self.items.offset((i - 1) as isize)).data as *mut T }
        } else {
            null_mut()
        }
    }

    //Slower, but don't need to call sort first.
    pub unsafe fn FindUnsorted(&mut self, key: *const U, storage: *mut *mut Data<T, U>) -> *mut T {
        let mut i: c_int;

        i = 0;
        while (i as c_uint) < self.numItems {
            if unsafe { (*self.items.add(i as usize)).key == *key } {
                unsafe {
                    *storage = self.items.add(i as usize);
                }
                return unsafe { &mut (*self.items.add(i as usize)).data as *mut T };
            }
            i += 1;
        }

        null_mut()
    }

    // returns the top item's data
    // and removes the item from the map
    pub unsafe fn Pop(&mut self) -> *mut T {
        let mut top: *mut T = null_mut();

        if self.numItems != 0 {
            top = unsafe { &mut (*self.items).data as *mut T };
            self.items = unsafe { self.items.add(1) };
            self.numItems -= 1;
        }

        top
    }

    pub unsafe extern "C" fn FixedMapSorter(a: *const c_void, b: *const c_void) -> c_int {
        if unsafe { (*(a as *mut Data<T, U>)).key > (*(b as *mut Data<T, U>)).key } {
            1
        } else if unsafe { (*(a as *mut Data<T, U>)).key == (*(b as *mut Data<T, U>)).key } {
            0
        } else {
            -1
        }
    }
}

impl<T: Copy, U: Copy + PartialOrd> Drop for VVFixedMap<T, U> {
    fn drop(&mut self) {
        unsafe {
            self.items = self.items.sub((self.maxItems - self.numItems) as usize);
            if !self.items.is_null() {
                let layout = Layout::array::<Data<T, U>>(self.maxItems as usize).unwrap();
                dealloc(self.items as *mut u8, layout);
            }
            self.numItems = 0;
        }
    }
}
