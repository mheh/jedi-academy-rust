// Filename:-	sstring.h
//
// Gil's string template, used to replace Microsoft's <string> vrsion which doesn't compile under certain stl map<>
//	conditions...

use core::ffi::c_char;

extern "C" {
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, size: usize, bOK: i32);
    fn strlen(s: *const c_char) -> usize;
    fn strcmpi(s1: *const c_char, s2: *const c_char) -> i32;
}

struct SStorage<const MaxSize: usize> {
    data: [c_char; MaxSize],
}

pub struct sstring<const MaxSize: usize> {
    mStorage: SStorage<MaxSize>,
}

impl<const MaxSize: usize> sstring<MaxSize> {
    /* don't figure we need this
    template<int oMaxSize>
    sstring(const sstring<oMaxSize> &o)
    {
        assert(strlen(o.mStorage.data)<MaxSize);
        strcpy(mStorage.data,o.mStorage.data);
    }
    */

    pub fn new_from_sstring(o: &sstring<MaxSize>) -> Self {
        let mut result = sstring {
            mStorage: SStorage {
                data: [0; MaxSize],
            },
        };
        //strcpy(mStorage.data,o.mStorage.data);
        unsafe {
            Q_strncpyz(
                result.mStorage.data.as_mut_ptr(),
                o.mStorage.data.as_ptr(),
                core::mem::size_of_val(&result.mStorage.data),
                1,
            );
        }
        result
    }

    pub fn new_from_str(s: *const c_char) -> Self {
        let mut result = sstring {
            mStorage: SStorage {
                data: [0; MaxSize],
            },
        };
        //assert(strlen(s)<MaxSize);
        //strcpy(mStorage.data,s);
        unsafe {
            Q_strncpyz(
                result.mStorage.data.as_mut_ptr(),
                s,
                core::mem::size_of_val(&result.mStorage.data),
                1,
            );
        }
        result
    }

    pub fn new() -> Self {
        let mut result = sstring {
            mStorage: SStorage {
                data: [0; MaxSize],
            },
        };
        unsafe {
            *result.mStorage.data.as_mut_ptr() = 0;
        }
        result
    }

    /* don't figure we need this
    template<int oMaxSize>
    sstring<oMaxSize> & operator =(const sstring<oMaxSize> &o)
    {
        assert(strlen(o.mStorage.data)<MaxSize);
        strcpy(mStorage.data,o.mStorage.data);
        return *this;
    }
    */

    pub fn operator_assign(&mut self, o: &sstring<MaxSize>) -> &mut Self {
        //strcpy(mStorage.data,o.mStorage.data);
        unsafe {
            Q_strncpyz(
                self.mStorage.data.as_mut_ptr(),
                o.mStorage.data.as_ptr(),
                core::mem::size_of_val(&self.mStorage.data),
                1,
            );
        }
        self
    }

    pub fn operator_assign_str(&mut self, s: *const c_char) -> &mut Self {
        unsafe {
            assert!(strlen(s) < MaxSize);
        }
        //strcpy(mStorage.data,s);
        unsafe {
            Q_strncpyz(
                self.mStorage.data.as_mut_ptr(),
                s,
                core::mem::size_of_val(&self.mStorage.data),
                1,
            );
        }
        self
    }

    pub fn c_str(&self) -> *const c_char {
        self.mStorage.data.as_ptr()
    }

    pub fn c_str_mut(&mut self) -> *mut c_char {
        self.mStorage.data.as_mut_ptr()
    }

    pub fn capacity(&self) -> i32 {
        MaxSize as i32	// not sure if this should be MaxSize-1? depends if talking bytes or strlen space I guess
    }

    pub fn length(&self) -> usize {
        unsafe { strlen(self.mStorage.data.as_ptr()) }
    }

    pub fn operator_eq(&self, o: &sstring<MaxSize>) -> bool {
        unsafe {
            if strcmpi(self.mStorage.data.as_ptr(), o.mStorage.data.as_ptr()) == 0 {
                return true;
            }
        }
        false
    }

    pub fn operator_ne(&self, o: &sstring<MaxSize>) -> bool {
        unsafe {
            if strcmpi(self.mStorage.data.as_ptr(), o.mStorage.data.as_ptr()) != 0 {
                return true;
            }
        }
        false
    }

    pub fn operator_lt(&self, o: &sstring<MaxSize>) -> bool {
        unsafe {
            if strcmpi(self.mStorage.data.as_ptr(), o.mStorage.data.as_ptr()) < 0 {
                return true;
            }
        }
        false
    }

    pub fn operator_gt(&self, o: &sstring<MaxSize>) -> bool {
        unsafe {
            if strcmpi(self.mStorage.data.as_ptr(), o.mStorage.data.as_ptr()) > 0 {
                return true;
            }
        }
        false
    }
}

pub type sstring_t = sstring<MAX_QPATH>;

/////////////////// eof ////////////////////
