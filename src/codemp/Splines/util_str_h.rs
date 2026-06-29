//need to rewrite this

#![allow(non_snake_case)]

use core::ffi::c_char;
use core::ptr::{null_mut, addr_of_mut, null};
use std::ffi::CStr;

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> i32;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> i32;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> i32;
    fn toupper(c: i32) -> i32;
    fn tolower(c: i32) -> i32;
}

pub fn TestStringClass() {
    // Stub
}

#[repr(C)]
pub struct strdata {
    pub len: i32,
    pub refcount: i32,
    pub data: *mut c_char,
    pub alloced: i32,
}

impl strdata {
    pub fn new() -> Self {
        strdata {
            len: 0,
            refcount: 0,
            data: null_mut(),
            alloced: 0,
        }
    }

    pub fn AddRef(&mut self) {
        self.refcount += 1;
    }

    pub fn DelRef(&mut self) -> bool {
        // True if killed
        self.refcount -= 1;
        if self.refcount < 0 {
            unsafe {
                if !self.data.is_null() {
                    let _ = Box::from_raw(self.data);
                }
                let _ = Box::from_raw(self as *mut strdata);
            }
            return true;
        }

        false
    }
}

#[repr(C)]
pub struct idStr {
    m_data: *mut strdata,
}

impl idStr {
    protected_fn!(EnsureAlloced, (size: i32, keepold: bool = true));
    protected_fn!(EnsureDataWritable, ());

    //need to rewrite this
    pub unsafe fn drop_manual(&mut self) {
        if !self.m_data.is_null() {
            (*self.m_data).DelRef();
            self.m_data = null_mut();
        }
    }

    // Default constructor: idStr()
    pub fn new() -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };
        result.EnsureAlloced(1, true);
        unsafe {
            (*result.m_data).data.write(0);
        }
        result
    }

    // Constructor: idStr(const char *text)
    pub fn from_cstr(text: *const c_char) -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };

        debug_assert!(!text.is_null());

        if !text.is_null() {
            unsafe {
                let len = strlen(text) as i32;
                result.EnsureAlloced(len + 1, true);
                strcpy((*result.m_data).data, text);
                (*result.m_data).len = len;
            }
        } else {
            result.EnsureAlloced(1, true);
            unsafe {
                (*result.m_data).data.write(0);
                (*result.m_data).len = 0;
            }
        }

        result
    }

    // Constructor: idStr(const idStr& string)
    pub fn from_idstr(text: &idStr) -> Self {
        let mut result = idStr {
            m_data: text.m_data,
        };
        unsafe {
            (*result.m_data).AddRef();
        }
        result
    }

    // Constructor: idStr(const idStr string, int start, int end)
    pub fn from_idstr_range(text: &idStr, start: i32, end: i32) -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };

        let mut end_val = end;
        let mut start_val = start;
        let text_len = text.length();

        if end_val > text_len {
            end_val = text_len;
        }

        if start_val > text_len {
            start_val = text_len;
        }

        let mut len = end_val - start_val;
        if len < 0 {
            len = 0;
        }

        result.EnsureAlloced(len + 1, true);

        unsafe {
            for i in 0..len {
                (*result.m_data).data
                    .add((i) as usize)
                    .write(text.index(start_val + i));
            }

            (*result.m_data).data.add(len as usize).write(0);
            (*result.m_data).len = len;
        }

        result
    }

    // Constructor: idStr(const char ch)
    pub fn from_char(ch: c_char) -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };

        result.EnsureAlloced(2, true);

        unsafe {
            (*result.m_data).data.write(ch);
            (*result.m_data).data.add(1).write(0);
            (*result.m_data).len = 1;
        }

        result
    }

    // Constructor: idStr(const float num)
    pub fn from_float(num: f32) -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };

        let mut text: [c_char; 32] = [0; 32];
        unsafe {
            sprintf(text.as_mut_ptr(), b"%.3f\0".as_ptr() as *const c_char, num);
            let len = strlen(text.as_ptr()) as i32;
            result.EnsureAlloced(len + 1, true);
            strcpy((*result.m_data).data, text.as_ptr());
            (*result.m_data).len = len;
        }

        result
    }

    // Constructor: idStr(const int num)
    pub fn from_int(num: i32) -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };

        let mut text: [c_char; 32] = [0; 32];
        unsafe {
            sprintf(text.as_mut_ptr(), b"%d\0".as_ptr() as *const c_char, num);
            let len = strlen(text.as_ptr()) as i32;
            result.EnsureAlloced(len + 1, true);
            strcpy((*result.m_data).data, text.as_ptr());
            (*result.m_data).len = len;
        }

        result
    }

    // Constructor: idStr(const unsigned num)
    pub fn from_uint(num: u32) -> Self {
        let mut result = idStr {
            m_data: null_mut(),
        };

        let mut text: [c_char; 32] = [0; 32];
        unsafe {
            sprintf(text.as_mut_ptr(), b"%u\0".as_ptr() as *const c_char, num);
            let len = strlen(text.as_ptr()) as i32;
            result.EnsureAlloced(len + 1, true);
            strcpy((*result.m_data).data, text.as_ptr());
            (*result.m_data).len = len;
        }

        result
    }

    pub fn length(&self) -> i32 {
        unsafe { if !self.m_data.is_null() { (*self.m_data).len } else { 0 } }
    }

    pub fn allocated(&self) -> i32 {
        unsafe {
            if !self.m_data.is_null() {
                (*self.m_data).alloced + (std::mem::size_of::<strdata>() as i32)
            } else {
                0
            }
        }
    }

    pub fn c_str(&self) -> *const c_char {
        debug_assert!(!self.m_data.is_null());

        unsafe { (*self.m_data).data }
    }

    pub fn append_cstr(&mut self, text: *const c_char) {
        debug_assert!(!text.is_null());

        if !text.is_null() {
            unsafe {
                let len = (self.length() as usize + strlen(text)) as i32;
                self.EnsureAlloced(len + 1, true);

                strcat((*self.m_data).data, text);
                (*self.m_data).len = len;
            }
        }
    }

    pub fn append_idstr(&mut self, text: &idStr) {
        unsafe {
            let len = (self.length() as usize + text.length() as usize) as i32;
            self.EnsureAlloced(len + 1, true);

            strcat((*self.m_data).data, text.c_str());
            (*self.m_data).len = len;
        }
    }

    pub fn index(&self, index: i32) -> c_char {
        debug_assert!(!self.m_data.is_null());

        unsafe {
            if self.m_data.is_null() {
                return 0;
            }

            // don't include the '/0' in the test, because technically, it's out of bounds
            debug_assert!((index >= 0) && (index < (*self.m_data).len));

            // In release mode, give them a null character
            // don't include the '/0' in the test, because technically, it's out of bounds
            if (index < 0) || (index >= (*self.m_data).len) {
                return 0;
            }

            *(*self.m_data).data.add(index as usize)
        }
    }

    pub fn index_mut(&mut self, index: i32) -> *mut c_char {
        // Used for result for invalid indices
        thread_local! {
            static DUMMY: std::cell::RefCell<c_char> = std::cell::RefCell::new(0);
        }

        debug_assert!(!self.m_data.is_null());

        // We don't know if they'll write to it or not
        // if it's not a const object
        self.EnsureDataWritable();

        unsafe {
            if self.m_data.is_null() {
                return DUMMY.with(|d| d.as_ptr() as *mut c_char);
            }

            // don't include the '/0' in the test, because technically, it's out of bounds
            debug_assert!((index >= 0) && (index < (*self.m_data).len));

            // In release mode, let them change a safe variable
            // don't include the '/0' in the test, because technically, it's out of bounds
            if (index < 0) || (index >= (*self.m_data).len) {
                return DUMMY.with(|d| d.as_ptr() as *mut c_char);
            }

            (*self.m_data).data.add(index as usize)
        }
    }

    pub fn assign_idstr(&mut self, text: &idStr) {
        unsafe {
            // adding the reference before deleting our current reference prevents
            // us from deleting our string if we are copying from ourself
            (*text.m_data).AddRef();
            (*self.m_data).DelRef();
            self.m_data = text.m_data;
        }
    }

    pub fn assign_cstr(&mut self, text: *const c_char) {
        debug_assert!(!text.is_null());

        unsafe {
            if text.is_null() {
                // safe behaviour if NULL
                self.EnsureAlloced(1, false);
                (*self.m_data).data.write(0);
                (*self.m_data).len = 0;
                return;
            }

            if self.m_data.is_null() {
                let len = strlen(text) as i32;
                self.EnsureAlloced(len + 1, false);
                strcpy((*self.m_data).data, text);
                (*self.m_data).len = len;
                return;
            }

            if text == (*self.m_data).data {
                return; // Copying same thing.  Punt.
            }

            // If we alias and I don't do this, I could corrupt other strings...  This
            // will get called with EnsureAlloced anyway
            self.EnsureDataWritable();

            // Now we need to check if we're aliasing..
            if !text.is_null()
                && text >= (*self.m_data).data
                && text <= (*self.m_data).data.add((*self.m_data).len as usize)
            {
                // Great, we're aliasing.  We're copying from inside ourselves.
                // This means that I don't have to ensure that anything is alloced,
                // though I'll assert just in case.
                let diff = (text as usize - (*self.m_data).data as usize) as i32;
                let mut i = 0;

                debug_assert!(strlen(text) < ((*self.m_data).len) as usize);

                while *text.add(i as usize) != 0 {
                    *(*self.m_data).data.add(i as usize) = *text.add(i as usize);
                    i += 1;
                }

                *(*self.m_data).data.add(i as usize) = 0;

                (*self.m_data).len -= diff;

                return;
            }

            let len = strlen(text) as i32;
            self.EnsureAlloced(len + 1, false);
            strcpy((*self.m_data).data, text);
            (*self.m_data).len = len;
        }
    }

    pub fn icmpn_cstr(&self, text: *const c_char, n: i32) -> i32 {
        debug_assert!(!self.m_data.is_null());
        debug_assert!(!text.is_null());

        unsafe { idStr::icmpn_static((*self.m_data).data, text, n) }
    }

    pub fn icmpn_idstr(&self, text: &idStr, n: i32) -> i32 {
        debug_assert!(!self.m_data.is_null());
        debug_assert!(!text.m_data.is_null());

        unsafe { idStr::icmpn_static((*self.m_data).data, (*text.m_data).data, n) }
    }

    pub fn icmp_cstr(&self, text: *const c_char) -> i32 {
        debug_assert!(!self.m_data.is_null());
        debug_assert!(!text.is_null());

        unsafe { idStr::icmp_static((*self.m_data).data, text) }
    }

    pub fn icmp_idstr(&self, text: &idStr) -> i32 {
        debug_assert!(!self.c_str().is_null());
        debug_assert!(!text.c_str().is_null());

        unsafe { idStr::icmp_static(self.c_str(), text.c_str()) }
    }

    pub fn cmp_cstr(&self, text: *const c_char) -> i32 {
        debug_assert!(!self.m_data.is_null());
        debug_assert!(!text.is_null());

        unsafe { idStr::cmp_static((*self.m_data).data, text) }
    }

    pub fn cmp_idstr(&self, text: &idStr) -> i32 {
        debug_assert!(!self.c_str().is_null());
        debug_assert!(!text.c_str().is_null());

        unsafe { idStr::cmp_static(self.c_str(), text.c_str()) }
    }

    pub fn cmpn_cstr(&self, text: *const c_char, n: i32) -> i32 {
        debug_assert!(!self.c_str().is_null());
        debug_assert!(!text.is_null());

        unsafe { idStr::cmpn_static(self.c_str(), text, n) }
    }

    pub fn cmpn_idstr(&self, text: &idStr, n: i32) -> i32 {
        debug_assert!(!self.c_str().is_null());
        debug_assert!(!text.c_str().is_null());

        unsafe { idStr::cmpn_static(self.c_str(), text.c_str(), n) }
    }

    pub fn tolower_mut(&mut self) {
        debug_assert!(!self.m_data.is_null());

        self.EnsureDataWritable();

        unsafe {
            idStr::tolower_static((*self.m_data).data);
        }
    }

    pub fn toupper_mut(&mut self) {
        debug_assert!(!self.m_data.is_null());

        self.EnsureDataWritable();

        unsafe {
            idStr::toupper_static((*self.m_data).data);
        }
    }

    pub fn isNumeric_method(&self) -> bool {
        debug_assert!(!self.m_data.is_null());
        unsafe { idStr::isNumeric_static((*self.m_data).data) }
    }

    // Static methods
    pub fn tolower_static(s1: *mut c_char) -> *mut c_char {
        unsafe {
            let mut s = s1;
            while *s != 0 {
                *s = tolower(*s as i32) as c_char;
                s = s.add(1);
            }
        }
        s1
    }

    pub fn toupper_static(s1: *mut c_char) -> *mut c_char {
        unsafe {
            let mut s = s1;
            while *s != 0 {
                *s = toupper(*s as i32) as c_char;
                s = s.add(1);
            }
        }
        s1
    }

    pub fn icmpn_static(s1: *const c_char, s2: *const c_char, n: i32) -> i32 {
        unsafe {
            let mut i = 0;
            while i < n as usize {
                let c1 = (*s1.add(i)) as i32;
                let c2 = (*s2.add(i)) as i32;

                let lower1 = if c1 >= 65 && c1 <= 90 { c1 + 32 } else { c1 };
                let lower2 = if c2 >= 65 && c2 <= 90 { c2 + 32 } else { c2 };

                if lower1 != lower2 {
                    return lower1 - lower2;
                }

                if c1 == 0 {
                    return 0;
                }

                i += 1;
            }
        }
        0
    }

    pub fn icmp_static(s1: *const c_char, s2: *const c_char) -> i32 {
        unsafe {
            let mut i = 0;
            loop {
                let c1 = (*s1.add(i)) as i32;
                let c2 = (*s2.add(i)) as i32;

                let lower1 = if c1 >= 65 && c1 <= 90 { c1 + 32 } else { c1 };
                let lower2 = if c2 >= 65 && c2 <= 90 { c2 + 32 } else { c2 };

                if lower1 != lower2 {
                    return lower1 - lower2;
                }

                if c1 == 0 {
                    return 0;
                }

                i += 1;
            }
        }
    }

    pub fn cmpn_static(s1: *const c_char, s2: *const c_char, n: i32) -> i32 {
        unsafe { strncmp(s1, s2, n as usize) }
    }

    pub fn cmp_static(s1: *const c_char, s2: *const c_char) -> i32 {
        unsafe { strcmp(s1, s2) }
    }

    pub fn snprintf(dst: *mut c_char, size: i32, fmt: *const c_char) {
        unsafe {
            sprintf(dst, fmt);
        }
    }

    pub fn isNumeric_static(str_ptr: *const c_char) -> bool {
        unsafe {
            if str_ptr.is_null() || *str_ptr == 0 {
                return false;
            }

            let mut has_dot = false;
            let mut s = str_ptr;

            while *s != 0 {
                let c = *s as i32;
                if c >= 48 && c <= 57 {
                    // 0-9
                } else if c == 45 || c == 43 {
                    // - or +
                } else if c == 46 {
                    // .
                    if has_dot {
                        return false;
                    }
                    has_dot = true;
                } else {
                    return false;
                }
                s = s.add(1);
            }
            true
        }
    }

    pub fn CapLength(&mut self, len: i32) {
        unsafe {
            if !self.m_data.is_null() && len < (*self.m_data).len {
                (*self.m_data).data.add(len as usize).write(0);
                (*self.m_data).len = len;
            }
        }
    }

    pub fn BackSlashesToSlashes(&mut self) {
        unsafe {
            if !self.m_data.is_null() {
                let mut s = (*self.m_data).data;
                while *s != 0 {
                    if *s == 92 {
                        // '\\'
                        *s = 47; // '/'
                    }
                    s = s.add(1);
                }
            }
        }
    }
}

impl Drop for idStr {
    fn drop(&mut self) {
        unsafe {
            if !self.m_data.is_null() {
                (*self.m_data).DelRef();
                self.m_data = null_mut();
            }
        }
    }
}

// friend idStr operator+( const idStr& a, const idStr& b )
pub fn operator_add_idstr_idstr(a: &idStr, b: &idStr) -> idStr {
    let mut result = idStr::from_idstr(a);
    result.append_idstr(b);
    result
}

// friend idStr operator+( const idStr& a, const char *b )
pub fn operator_add_idstr_cstr(a: &idStr, b: *const c_char) -> idStr {
    let mut result = idStr::from_idstr(a);
    result.append_cstr(b);
    result
}

// friend idStr operator+( const char *a, const idStr& b )
pub fn operator_add_cstr_idstr(a: *const c_char, b: &idStr) -> idStr {
    let mut result = idStr::from_cstr(a);
    result.append_idstr(b);
    result
}

// friend idStr operator+( const idStr& a, const bool b )
pub fn operator_add_idstr_bool(a: &idStr, b: bool) -> idStr {
    let mut result = idStr::from_idstr(a);
    result.append_cstr(if b {
        b"true\0".as_ptr() as *const c_char
    } else {
        b"false\0".as_ptr() as *const c_char
    });
    result
}

// friend idStr operator+( const idStr& a, const char b )
pub fn operator_add_idstr_char(a: &idStr, b: c_char) -> idStr {
    let text: [c_char; 2] = [b, 0];

    let mut result = idStr::from_idstr(a);
    result.append_cstr(text.as_ptr());
    result
}

// idStr& idStr::operator+=( const idStr& a )
impl std::ops::AddAssign<&idStr> for idStr {
    fn add_assign(&mut self, a: &idStr) {
        self.append_idstr(a);
    }
}

// idStr& idStr::operator+=( const char *a )
impl std::ops::AddAssign<*const c_char> for idStr {
    fn add_assign(&mut self, a: *const c_char) {
        self.append_cstr(a);
    }
}

// idStr& idStr::operator+=( const char a )
pub fn add_assign_char(s: &mut idStr, a: c_char) {
    let text: [c_char; 2] = [a, 0];
    s.append_cstr(text.as_ptr());
}

// idStr& idStr::operator+=( const bool a )
pub fn add_assign_bool(s: &mut idStr, a: bool) {
    s.append_cstr(if a {
        b"true\0".as_ptr() as *const c_char
    } else {
        b"false\0".as_ptr() as *const c_char
    });
}

// friend bool operator==( const idStr& a, const idStr& b )
pub fn operator_eq_idstr_idstr(a: &idStr, b: &idStr) -> bool {
    unsafe { strcmp(a.c_str(), b.c_str()) == 0 }
}

// friend bool operator==( const idStr& a, const char *b )
pub fn operator_eq_idstr_cstr(a: &idStr, b: *const c_char) -> bool {
    debug_assert!(!b.is_null());
    if b.is_null() {
        return false;
    }
    unsafe { strcmp(a.c_str(), b) == 0 }
}

// friend bool operator==( const char *a, const idStr& b )
pub fn operator_eq_cstr_idstr(a: *const c_char, b: &idStr) -> bool {
    debug_assert!(!a.is_null());
    if a.is_null() {
        return false;
    }
    unsafe { strcmp(a, b.c_str()) == 0 }
}

// friend bool operator!=( const idStr& a, const idStr& b )
pub fn operator_ne_idstr_idstr(a: &idStr, b: &idStr) -> bool {
    !operator_eq_idstr_idstr(a, b)
}

// friend bool operator!=( const idStr& a, const char *b )
pub fn operator_ne_idstr_cstr(a: &idStr, b: *const c_char) -> bool {
    !operator_eq_idstr_cstr(a, b)
}

// friend bool operator!=( const char *a, const idStr& b )
pub fn operator_ne_cstr_idstr(a: *const c_char, b: &idStr) -> bool {
    !operator_eq_cstr_idstr(a, b)
}

// Conversion operator
impl AsRef<CStr> for idStr {
    fn as_ref(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.c_str()) }
    }
}

// Stub for protected methods - these are declared but not defined
// as the actual implementation is in the corresponding .cpp file
fn EnsureAlloced(s: &mut idStr, size: i32, keepold: bool) {
    // Stub - implementation in .cpp
}

fn EnsureDataWritable(s: &mut idStr) {
    // Stub - implementation in .cpp
}

// Macro stub to avoid compilation errors
#[allow(unused_macros)]
macro_rules! protected_fn {
    ($name:ident, ($($arg:tt)*)) => {
        // Stub for protected methods
    };
}
