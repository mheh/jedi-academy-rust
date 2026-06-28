#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int, c_long};
use core::marker::PhantomData;
use core::ptr;

// C origin: `codemp/qcommon/chash.h`.

// Notes
// Make sure extension is stripped if it needs to be

// Template class must have
// 1. A GetName() accessor - a null terminated string case insensitive
// 2. A Destroy() function - normally "delete this"
// 3. SetNext(T *) and T *GetNext() functions

pub const HASH_SIZE: usize = 1024;

unsafe extern "C" {
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

pub unsafe trait CHashItem {
    unsafe fn GetName(item: *mut Self) -> *const c_char;
    unsafe fn Destroy(item: *mut Self);
    unsafe fn SetNext(item: *mut Self, next: *mut Self);
    unsafe fn GetNext(item: *mut Self) -> *mut Self;
}

pub trait CHashComparator {
    unsafe fn TCompare(s1: *const c_char, s2: *const c_char) -> c_int;
}

pub struct strcmp_compare;

impl CHashComparator for strcmp_compare {
    unsafe fn TCompare(s1: *const c_char, s2: *const c_char) -> c_int {
        unsafe { strcmp(s1, s2) }
    }
}

pub struct CHash<T, const TSize: usize = HASH_SIZE, TCompare = strcmp_compare>
where
    T: CHashItem,
    TCompare: CHashComparator,
{
    mHashTable: [*mut T; TSize],
    mNext: *mut T,
    mCount: c_int,
    mPrevious: *mut T, // Internal work variable
    mHash: c_long,    // Internal work variable
    _TCompare: PhantomData<TCompare>,
}

impl<T, const TSize: usize, TCompare> CHash<T, TSize, TCompare>
where
    T: CHashItem,
    TCompare: CHashComparator,
{
    // Creates the hash value and sets the mHash member
    unsafe fn CreateHash(&mut self, mut key: *const c_char) {
        let mut i: c_int = 0;
        let mut letter: c_char;

        self.mHash = 0;
        letter = unsafe { *key };
        key = unsafe { key.add(1) };
        while letter != 0 {
            self.mHash += (letter as c_long).wrapping_mul((i + 119) as c_long);

            i += 1;
            letter = unsafe { *key };
            key = unsafe { key.add(1) };
        }
        self.mHash &= (TSize as c_long).wrapping_sub(1);
    }

    // Constructor
    pub fn CHash() -> Self {
        Self {
            mHashTable: [ptr::null_mut(); TSize],
            mNext: ptr::null_mut(),
            mCount: 0,
            mPrevious: ptr::null_mut(),
            mHash: 0,
            _TCompare: PhantomData,
        }
    }

    // Returns the total number of entries in the hash table
    pub fn count(&self) -> c_int {
        self.mCount
    }

    // Inserts an item into the hash table
    pub unsafe fn insert(&mut self, item: *mut T) {
        unsafe { self.CreateHash(T::GetName(item)) };
        unsafe { T::SetNext(item, self.mHashTable[self.mHash as usize]) };
        self.mHashTable[self.mHash as usize] = item;
        self.mCount += 1;
    }

    // Finds an item in the hash table (sets the mPrevious member)
    pub unsafe fn find(&mut self, key: *const c_char) -> *mut T {
        unsafe { self.CreateHash(key) };
        let mut item: *mut T = self.mHashTable[self.mHash as usize];
        self.mPrevious = ptr::null_mut();
        while !item.is_null() {
            self.mNext = unsafe { T::GetNext(item) };
            if unsafe { TCompare::TCompare(T::GetName(item), key) } == 0 {
                return item;
            }
            self.mPrevious = item;
            item = self.mNext;
        }
        ptr::null_mut()
    }

    // Remove item from the hash table referenced by key
    pub unsafe fn remove(&mut self, key: *const c_char) -> bool {
        let item: *mut T = unsafe { self.find(key) };
        if !item.is_null() {
            let next: *mut T = unsafe { T::GetNext(item) };
            if !self.mPrevious.is_null() {
                unsafe { T::SetNext(self.mPrevious, next) };
            } else {
                self.mHashTable[self.mHash as usize] = next;
            }
            unsafe { T::Destroy(item) };
            self.mCount -= 1;
            return true;
        }
        false
    }

    // Remove item from hash referenced by item
    pub unsafe fn remove_item(&mut self, item: *mut T) -> bool {
        unsafe { self.remove(T::GetName(item)) }
    }

    // Returns the first valid entry
    pub unsafe fn head(&mut self) -> *mut T {
        self.mHash = -1;
        self.mNext = ptr::null_mut();
        unsafe { self.next() }
    }

    // Returns the next entry in the hash table
    pub unsafe fn next(&mut self) -> *mut T {
        let mut item: *mut T;

        assert!(self.mHash < TSize as c_long);

        if !self.mNext.is_null() {
            item = self.mNext;
            self.mNext = unsafe { T::GetNext(item) };
            return item;
        }
        self.mHash += 1;

        while self.mHash < TSize as c_long {
            item = self.mHashTable[self.mHash as usize];
            if !item.is_null() {
                self.mNext = unsafe { T::GetNext(item) };
                return item;
            }
            self.mHash += 1;
        }
        ptr::null_mut()
    }

    // Destroy all entries in the hash table
    pub unsafe fn clear(&mut self) {
        let mut item: *mut T = unsafe { self.head() };
        while !item.is_null() {
            unsafe { self.remove_item(item) };
            item = unsafe { self.next() };
        }
    }

    // Override the [] operator
    pub unsafe fn operator_index(&mut self, key: *const c_char) -> *mut T {
        unsafe { self.find(key) }
    }
}

impl<T, const TSize: usize, TCompare> Drop for CHash<T, TSize, TCompare>
where
    T: CHashItem,
    TCompare: CHashComparator,
{
    // Destructor
    fn drop(&mut self) {
        unsafe { self.clear() };
    }
}

// end
