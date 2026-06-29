#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_long};
use core::ptr;

// Notes
// Make sure extension is stripped if it needs to be

// Template class must have
// 1. A GetName() accessor - a null terminated string case insensitive
// 2. A Destroy() function - normally "delete this"
// 3. SetNext(T *) and T *GetNext() functions

const HASH_SIZE: usize = 1024;

// Local stub for the template contract
pub trait CHashNode {
    unsafe fn GetName(&self) -> *const c_char;
    unsafe fn SetNext(&mut self, next: *mut Self);
    unsafe fn GetNext(&self) -> *mut Self;
    unsafe fn Destroy(&mut self);
}

pub struct CHash<T: CHashNode + ?Sized, const TSIZE: usize> {
    mHashTable: [*mut T; TSIZE],
    mNext: *mut T,
    mCount: c_int,
    mPrevious: *mut T,  // Internal work variable
    mHash: c_long,      // Internal work variable
}

impl<T: CHashNode + ?Sized, const TSIZE: usize> CHash<T, TSIZE> {
    // Creates the hash value and sets the mHash member
    fn CreateHash(&mut self, key: *const c_char) {
        let mut i: c_int = 0;
        let mut letter: c_char;

        self.mHash = 0;
        letter = unsafe { *key };
        let mut key_ptr = key;
        key_ptr = unsafe { key_ptr.add(1) };
        while letter != 0 {
            self.mHash += (letter as c_long) * ((i + 119) as c_long);

            i += 1;
            letter = unsafe { *key_ptr };
            key_ptr = unsafe { key_ptr.add(1) };
        }
        self.mHash &= (TSIZE - 1) as c_long;
    }

    // Constructor
    pub fn new() -> Self {
        CHash {
            mHashTable: [ptr::null_mut(); TSIZE],
            mNext: ptr::null_mut(),
            mCount: 0,
            mPrevious: ptr::null_mut(),
            mHash: 0,
        }
    }

    // Returns the total number of entries in the hash table
    pub fn count(&self) -> c_int {
        self.mCount
    }

    // Inserts an item into the hash table
    pub unsafe fn insert(&mut self, item: *mut T) {
        self.CreateHash((*item).GetName());
        (*item).SetNext(self.mHashTable[self.mHash as usize]);
        self.mHashTable[self.mHash as usize] = item;
        self.mCount += 1;
    }

    // Finds an item in the hash table (sets the mPrevious member)
    pub unsafe fn find(&mut self, key: *const c_char) -> *mut T {
        self.CreateHash(key);
        let mut item = self.mHashTable[self.mHash as usize];
        self.mPrevious = ptr::null_mut();
        while !item.is_null() {
            self.mNext = (*item).GetNext();
            if c_str_cmp((*item).GetName(), key) == 0 {
                return item;
            }
            self.mPrevious = item;
            item = self.mNext;
        }
        ptr::null_mut()
    }

    // Remove item from the hash table referenced by key
    pub unsafe fn remove(&mut self, key: *const c_char) -> bool {
        let item = self.find(key);
        if !item.is_null() {
            let next = (*item).GetNext();
            if !self.mPrevious.is_null() {
                (*self.mPrevious).SetNext(next);
            } else {
                self.mHashTable[self.mHash as usize] = next;
            }
            (*item).Destroy();
            self.mCount -= 1;
            return true;
        }
        false
    }

    // Remove item from hash referenced by item
    pub unsafe fn remove_item(&mut self, item: *mut T) -> bool {
        self.remove((*item).GetName())
    }

    // Returns the first valid entry
    pub unsafe fn head(&mut self) -> *mut T {
        self.mHash = -1;
        self.mNext = ptr::null_mut();
        self.next()
    }

    // Returns the next entry in the hash table
    pub unsafe fn next(&mut self) -> *mut T {
        let mut item: *mut T;

        assert!((self.mHash as usize) < TSIZE);

        if !self.mNext.is_null() {
            item = self.mNext;
            self.mNext = (*item).GetNext();
            return item;
        }
        self.mHash += 1;

        while (self.mHash as usize) < TSIZE {
            item = self.mHashTable[self.mHash as usize];
            if !item.is_null() {
                self.mNext = (*item).GetNext();
                return item;
            }
            self.mHash += 1;
        }
        ptr::null_mut()
    }

    // Destroy all entries in the hash table
    pub unsafe fn clear(&mut self) {
        let mut item = self.head();
        while !item.is_null() {
            self.remove((*item).GetName());
            item = self.next();
        }
    }

    // Override the [] operator
    pub unsafe fn index(&mut self, key: *const c_char) -> *mut T {
        self.find(key)
    }
}

impl<T: CHashNode + ?Sized, const TSIZE: usize> Drop for CHash<T, TSIZE> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            //		Com_OPrintf("Shutting down %s hash table .....", typeid(T).name());
        }
        unsafe { self.clear(); }
        #[cfg(debug_assertions)]
        {
            //		Com_OPrintf(" done\n");
        }
    }
}

// Local stub for strcmp (default TCompare)
fn c_str_cmp(a: *const c_char, b: *const c_char) -> c_int {
    unsafe {
        let mut a_ptr = a;
        let mut b_ptr = b;
        loop {
            let ca = *a_ptr;
            let cb = *b_ptr;
            if ca != cb {
                return (ca as i32) - (cb as i32);
            }
            if ca == 0 {
                return 0;
            }
            a_ptr = a_ptr.add(1);
            b_ptr = b_ptr.add(1);
        }
    }
}

// end
