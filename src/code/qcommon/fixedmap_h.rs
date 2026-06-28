#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use std::alloc::{alloc, dealloc, Layout};
use std::mem;

extern "C" {
    pub fn qsort(
        base: *mut c_void,
        nmemb: usize,
        size: usize,
        compar: extern "C" fn(*const c_void, *const c_void) -> c_int,
    );
}

/*
   An STL map-like container.  Quickly thrown together to replace STL maps
   in specific instances.  Many gotchas.  Use with caution.
*/

struct Data<T, U> {
    data: T,
    key: U,
}

// Helper comparison function for qsort
// Mirrors the C++ static FixedMapSorter method
fn fixedmap_sorter<T, U: Ord>(a: *const c_void, b: *const c_void) -> c_int {
    unsafe {
        if (*(a as *const Data<T, U>)).key > (*(b as *const Data<T, U>)).key {
            return 1;
        } else if (*(a as *const Data<T, U>)).key == (*(b as *const Data<T, U>)).key {
            return 0;
        } else {
            return -1;
        }
    }
}

pub struct VVFixedMap<T, U> {
    items: *mut Data<T, U>,
    original_items: *mut Data<T, U>,
    numItems: usize,
    maxItems: usize,
}

impl<T, U> VVFixedMap<T, U> {
    pub fn new(maxItems: usize) -> Self {
        unsafe {
            let layout = Layout::array::<Data<T, U>>(maxItems).expect("invalid layout");
            let ptr = alloc(layout) as *mut Data<T, U>;

            VVFixedMap {
                items: ptr,
                original_items: ptr,
                numItems: 0,
                maxItems,
            }
        }
    }

    pub fn Insert(&mut self, newItem: &T, key: &U) -> bool
    where
        T: Clone,
        U: Clone + PartialEq,
    {
        let mut storage: *mut Data<T, U> = std::ptr::null_mut();

        // Check for fullness.
        if self.numItems >= self.maxItems {
            // assert!( 0 );
            return false;
        }

        // Check for reuse.
        if !self.FindUnsorted(key, &mut storage) {
            storage = unsafe { self.items.add(self.numItems) };
            self.numItems += 1;
        }

        unsafe {
            (*storage).data = newItem.clone();
            (*storage).key = key.clone();
        }

        true
    }

    pub fn Sort(&mut self)
    where
        U: Ord,
    {
        unsafe {
            qsort(
                self.items as *mut c_void,
                self.numItems,
                mem::size_of::<Data<T, U>>(),
                fixedmap_sorter::<T, U> as extern "C" fn(*const c_void, *const c_void) -> c_int,
            );
        }
    }

    // Binary search, items must have been sorted!
    pub fn Find(&self, key: &U) -> *mut T
    where
        U: PartialOrd + PartialEq,
    {
        let mut i: usize;
        let mut high: i32 = self.numItems as i32;
        let mut low: i32 = -1;

        loop {
            if high - low <= 1 {
                break;
            }
            i = ((high + low) / 2) as usize;
            unsafe {
                if key < &(*self.items.add(i)).key {
                    high = i as i32;
                } else if key > &(*self.items.add(i)).key {
                    low = i as i32;
                } else {
                    return &mut (*self.items.add(i)).data as *mut T;
                }
            }
        }

        unsafe {
            if (high + 1) as usize < self.numItems && (*self.items.add((high + 1) as usize)).key == *key
            {
                return &mut (*self.items.add((high + 1) as usize)).data as *mut T;
            } else if (high - 1) as usize < self.numItems
                && (*self.items.add((high - 1) as usize)).key == *key
            {
                return &mut (*self.items.add((high - 1) as usize)).data as *mut T;
            }
        }

        std::ptr::null_mut()
    }

    // Slower, but don't need to call sort first.
    pub fn FindUnsorted(&self, key: &U, storage: &mut *mut Data<T, U>) -> bool
    where
        U: PartialEq,
    {
        unsafe {
            for i in 0..self.numItems {
                if (*self.items.add(i)).key == *key {
                    *storage = self.items.add(i);
                    return true;
                }
            }
        }

        false
    }

    // returns the top item's data
    // and removes the item from the map
    pub fn Pop(&mut self) -> *mut T {
        let mut top: *mut T = std::ptr::null_mut();

        if self.numItems > 0 {
            unsafe {
                top = &mut (*self.items).data as *mut T;
                self.items = self.items.add(1);
                self.numItems -= 1;
            }
        }

        top
    }

    #[allow(non_snake_case)]
    fn FixedMapSorter(a: *const c_void, b: *const c_void) -> c_int
    where
        U: Ord,
    {
        unsafe {
            if (*(a as *const Data<T, U>)).key > (*(b as *const Data<T, U>)).key {
                return 1;
            } else if (*(a as *const Data<T, U>)).key == (*(b as *const Data<T, U>)).key {
                return 0;
            } else {
                return -1;
            }
        }
    }
}

impl<T, U> Drop for VVFixedMap<T, U> {
    fn drop(&mut self) {
        unsafe {
            // items -= ( maxItems - numItems );
            self.items = self.items.sub(self.maxItems - self.numItems);

            let layout = Layout::array::<Data<T, U>>(self.maxItems).expect("invalid layout");
            dealloc(self.items as *mut u8, layout);
            self.numItems = 0;
        }
    }
}
