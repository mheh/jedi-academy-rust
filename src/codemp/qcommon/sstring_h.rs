// Filename:-	sstring.h
//
// Gil's string template, used to replace Microsoft's <string> vrsion which doesn't compile under certain stl map<>
//	conditions...

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};

use crate::codemp::game::q_shared::{Q_stricmp, Q_strncpyz};
use crate::codemp::game::q_shared_h::MAX_QPATH;

unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

#[repr(C)]
pub struct SStorage<const MaxSize: usize> {
    pub data: [c_char; MaxSize],
}

#[repr(C)]
pub struct sstring<const MaxSize: usize> {
    pub mStorage: SStorage<MaxSize>,
}

impl<const MaxSize: usize> sstring<MaxSize> {
    /*
    don't figure we need this
        template<int oMaxSize>
        sstring(const sstring<oMaxSize> &o)
        {
            assert(strlen(o.mStorage.data)<MaxSize);
            strcpy(mStorage.data,o.mStorage.data);
        }
    */

    // Porting deviation: Rust cannot overload C++ constructors, so the copy
    // constructor is named explicitly.
    pub unsafe fn sstring_copy(o: *const sstring<MaxSize>) -> Self {
        let mut out = Self::sstring();
        //strcpy(mStorage.data,o.mStorage.data);
        unsafe {
            Q_strncpyz(
                out.mStorage.data.as_mut_ptr(),
                (*o).mStorage.data.as_ptr(),
                core::mem::size_of_val(&out.mStorage.data) as c_int,
            );
        }
        out
    }

    // Porting deviation: Rust cannot overload C++ constructors, so the
    // `const char *` constructor is named explicitly.
    pub unsafe fn sstring_char(s: *const c_char) -> Self {
        let mut out = Self::sstring();
        //assert(strlen(s)<MaxSize);
        //strcpy(mStorage.data,s);
        unsafe {
            Q_strncpyz(
                out.mStorage.data.as_mut_ptr(),
                s,
                core::mem::size_of_val(&out.mStorage.data) as c_int,
            );
        }
        out
    }

    pub fn sstring() -> Self {
        let mut out = Self {
            mStorage: SStorage { data: [0; MaxSize] },
        };
        out.mStorage.data[0] = 0;
        out
    }

    /*
    don't figure we need this
        template<int oMaxSize>
        sstring<oMaxSize> & operator =(const sstring<oMaxSize> &o)
        {
            assert(strlen(o.mStorage.data)<MaxSize);
            strcpy(mStorage.data,o.mStorage.data);
            return *this;
        }
    */

    pub unsafe fn operator_assign(&mut self, o: *const sstring<MaxSize>) -> *mut sstring<MaxSize> {
        //strcpy(mStorage.data,o.mStorage.data);
        unsafe {
            Q_strncpyz(
                self.mStorage.data.as_mut_ptr(),
                (*o).mStorage.data.as_ptr(),
                core::mem::size_of_val(&self.mStorage.data) as c_int,
            );
        }
        self as *mut sstring<MaxSize>
    }

    pub unsafe fn operator_assign_char(&mut self, s: *const c_char) -> *mut sstring<MaxSize> {
        assert!(unsafe { strlen(s) } < MaxSize);
        //strcpy(mStorage.data,s);
        unsafe {
            Q_strncpyz(
                self.mStorage.data.as_mut_ptr(),
                s,
                core::mem::size_of_val(&self.mStorage.data) as c_int,
            );
        }
        self as *mut sstring<MaxSize>
    }

    pub fn c_str(&mut self) -> *mut c_char {
        self.mStorage.data.as_mut_ptr()
    }

    // Porting deviation: Rust cannot overload on constness, so the const
    // overload is named explicitly.
    pub fn c_str_const(&self) -> *const c_char {
        self.mStorage.data.as_ptr()
    }

    pub fn capacity(&self) -> c_int {
        MaxSize as c_int
    }

    pub unsafe fn length(&self) -> c_int {
        unsafe { strlen(self.mStorage.data.as_ptr()) as c_int }
    }

    pub unsafe fn operator_eq(&self, o: *const sstring<MaxSize>) -> bool {
        if unsafe { Q_stricmp(self.mStorage.data.as_ptr(), (*o).mStorage.data.as_ptr()) } == 0 {
            return true;
        }
        false
    }

    pub unsafe fn operator_ne(&self, o: *const sstring<MaxSize>) -> bool {
        if unsafe { Q_stricmp(self.mStorage.data.as_ptr(), (*o).mStorage.data.as_ptr()) } != 0 {
            return true;
        }
        false
    }

    pub unsafe fn operator_lt(&self, o: *const sstring<MaxSize>) -> bool {
        if unsafe { Q_stricmp(self.mStorage.data.as_ptr(), (*o).mStorage.data.as_ptr()) } < 0 {
            return true;
        }
        false
    }

    pub unsafe fn operator_gt(&self, o: *const sstring<MaxSize>) -> bool {
        if unsafe { Q_stricmp(self.mStorage.data.as_ptr(), (*o).mStorage.data.as_ptr()) } > 0 {
            return true;
        }
        false
    }
}

pub type sstring_t = sstring<MAX_QPATH>;

/////////////////// eof ////////////////////
