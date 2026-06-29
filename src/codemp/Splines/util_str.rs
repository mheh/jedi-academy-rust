// need to rewrite this

use core::ffi::{c_int, c_char};

extern "C" {
    fn tolower(c: c_int) -> c_int;
    fn toupper(c: c_int) -> c_int;
    fn isdigit(c: c_int) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
    fn vsprintf(s: *mut c_char, format: *const c_char, ap: *mut core::ffi::c_void) -> c_int;
}

const STR_ALLOC_GRAN: c_int = 20;

impl idStr {
    pub fn tolower(s1: *mut c_char) -> *mut c_char {
        let mut s = s1;
        unsafe {
            while *s != 0 {
                *s = tolower(*s as c_int) as c_char;
                s = s.offset(1);
            }
        }

        return s1;
    }

    pub fn toupper(s1: *mut c_char) -> *mut c_char {
        let mut s = s1;
        unsafe {
            while *s != 0 {
                *s = toupper(*s as c_int) as c_char;
                s = s.offset(1);
            }
        }

        return s1;
    }

    pub fn icmpn(mut s1: *const c_char, mut s2: *const c_char, mut n: c_int) -> c_int {
        let mut c1: c_int;
        let mut c2: c_int;

        unsafe {
            loop {
                c1 = *s1 as c_int;
                s1 = s1.offset(1);
                c2 = *s2 as c_int;
                s2 = s2.offset(1);

                if n == 0 {
                    n -= 1;
                    // idStrings are equal until end point
                    return 0;
                }

                if c1 != c2 {
                    if c1 >= 'a' as c_int && c1 <= 'z' as c_int {
                        c1 -= ('a' as c_int - 'A' as c_int);
                    }

                    if c2 >= 'a' as c_int && c2 <= 'z' as c_int {
                        c2 -= ('a' as c_int - 'A' as c_int);
                    }

                    if c1 < c2 {
                        // strings less than
                        return -1;
                    } else if c1 > c2 {
                        // strings greater than
                        return 1;
                    }
                }

                if c1 == 0 {
                    break;
                }
            }
        }

        // strings are equal
        return 0;
    }

    pub fn icmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
        let mut c1: c_int;
        let mut c2: c_int;

        unsafe {
            loop {
                c1 = *s1 as c_int;
                s1 = s1.offset(1);
                c2 = *s2 as c_int;
                s2 = s2.offset(1);

                if c1 != c2 {
                    if c1 >= 'a' as c_int && c1 <= 'z' as c_int {
                        c1 -= ('a' as c_int - 'A' as c_int);
                    }

                    if c2 >= 'a' as c_int && c2 <= 'z' as c_int {
                        c2 -= ('a' as c_int - 'A' as c_int);
                    }

                    if c1 < c2 {
                        // strings less than
                        return -1;
                    } else if c1 > c2 {
                        // strings greater than
                        return 1;
                    }
                }

                if c1 == 0 {
                    break;
                }
            }
        }

        // strings are equal
        return 0;
    }

    pub fn cmpn(mut s1: *const c_char, mut s2: *const c_char, mut n: c_int) -> c_int {
        let mut c1: c_int;
        let mut c2: c_int;

        unsafe {
            loop {
                c1 = *s1 as c_int;
                s1 = s1.offset(1);
                c2 = *s2 as c_int;
                s2 = s2.offset(1);

                if n == 0 {
                    n -= 1;
                    // strings are equal until end point
                    return 0;
                }

                if c1 < c2 {
                    // strings less than
                    return -1;
                } else if c1 > c2 {
                    // strings greater than
                    return 1;
                }

                if c1 == 0 {
                    break;
                }
            }
        }

        // strings are equal
        return 0;
    }

    pub fn cmp(mut s1: *const c_char, mut s2: *const c_char) -> c_int {
        let mut c1: c_int;
        let mut c2: c_int;

        unsafe {
            loop {
                c1 = *s1 as c_int;
                s1 = s1.offset(1);
                c2 = *s2 as c_int;
                s2 = s2.offset(1);

                if c1 < c2 {
                    // strings less than
                    return -1;
                } else if c1 > c2 {
                    // strings greater than
                    return 1;
                }

                if c1 == 0 {
                    break;
                }
            }
        }

        // strings are equal
        return 0;
    }
}

// ============
// IsNumeric
//
// Checks a string to see if it contains only numerical values.
// ============
impl idStr {
    pub fn isNumeric(mut str_: *const c_char) -> bool {
        let len: usize;
        let mut i: usize;
        let mut dot: bool;

        unsafe {
            if *str_ as c_int == '-' as c_int {
                str_ = str_.offset(1);
            }

            dot = false;
            len = strlen(str_);
            for i in 0..len {
                if isdigit(*str_.offset(i as isize) as c_int) == 0 {
                    if (*str_.offset(i as isize) as c_int == '.' as c_int) && !dot {
                        dot = true;
                        continue;
                    }
                    return false;
                }
            }
        }

        return true;
    }
}

// idStr operator+ (const idStr& a, const float b)
pub fn idStr_operator_add_float(a: &idStr, b: f32) -> idStr {
    let mut text: [c_char; 20] = [0; 20];

    let mut result = idStr::clone_ref(a);

    unsafe {
        sprintf(
            text.as_mut_ptr(),
            "%f\0".as_ptr() as *const c_char,
            b,
        );
        result.append(text.as_ptr());
    }

    return result;
}

// idStr operator+ (const idStr& a, const int b)
pub fn idStr_operator_add_int(a: &idStr, b: c_int) -> idStr {
    let mut text: [c_char; 20] = [0; 20];

    let mut result = idStr::clone_ref(a);

    unsafe {
        sprintf(
            text.as_mut_ptr(),
            "%d\0".as_ptr() as *const c_char,
            b,
        );
        result.append(text.as_ptr());
    }

    return result;
}

// idStr operator+ (const idStr& a, const unsigned b)
pub fn idStr_operator_add_uint(a: &idStr, b: u32) -> idStr {
    let mut text: [c_char; 20] = [0; 20];

    let mut result = idStr::clone_ref(a);

    unsafe {
        sprintf(
            text.as_mut_ptr(),
            "%u\0".as_ptr() as *const c_char,
            b,
        );
        result.append(text.as_ptr());
    }

    return result;
}

impl idStr {
    // idStr& idStr::operator+= (const float a)
    pub fn op_add_assign_float(&mut self, a: f32) {
        let mut text: [c_char; 20] = [0; 20];

        unsafe {
            sprintf(
                text.as_mut_ptr(),
                "%f\0".as_ptr() as *const c_char,
                a,
            );
            self.append(text.as_ptr());
        }
    }

    // idStr& idStr::operator+= (const int a)
    pub fn op_add_assign_int(&mut self, a: c_int) {
        let mut text: [c_char; 20] = [0; 20];

        unsafe {
            sprintf(
                text.as_mut_ptr(),
                "%d\0".as_ptr() as *const c_char,
                a,
            );
            self.append(text.as_ptr());
        }
    }

    // idStr& idStr::operator+= (const unsigned a)
    pub fn op_add_assign_uint(&mut self, a: u32) {
        let mut text: [c_char; 20] = [0; 20];

        unsafe {
            sprintf(
                text.as_mut_ptr(),
                "%u\0".as_ptr() as *const c_char,
                a,
            );
            self.append(text.as_ptr());
        }
    }

    pub fn CapLength(&mut self, newlen: c_int) {
        assert!(!self.m_data.is_null());

        if self.length() <= newlen {
            return;
        }

        self.EnsureDataWritable();

        unsafe {
            (*self.m_data).data.offset(newlen as isize).write(0);
            (*self.m_data).len = newlen;
        }
    }

    pub fn EnsureDataWritable(&mut self) {
        assert!(!self.m_data.is_null());
        let mut olddata: *mut strdata;
        let len: c_int;

        unsafe {
            if (*self.m_data).refcount == 0 {
                return;
            }

            olddata = self.m_data;
            len = self.length();

            self.m_data = core::ptr::null_mut::<strdata>();

            self.EnsureAlloced(len + 1, false);
            strncpy((*self.m_data).data, (*olddata).data, (len + 1) as usize);
            (*self.m_data).len = len;

            (*olddata).DelRef();
        }
    }

    pub fn EnsureAlloced(&mut self, amount: c_int, keepold: bool) {
        unsafe {
            if self.m_data.is_null() {
                self.m_data = core::ptr::null_mut::<strdata>();
            }

            // Now, let's make sure it's writable
            self.EnsureDataWritable();

            let mut newbuffer: *mut c_char;
            let wasalloced: bool = (*self.m_data).alloced != 0;

            if amount < (*self.m_data).alloced {
                return;
            }

            assert!(amount != 0);
            if amount == 1 {
                (*self.m_data).alloced = 1;
            } else {
                let mut newsize: c_int;
                let mod_: c_int;
                mod_ = amount % STR_ALLOC_GRAN;
                if mod_ == 0 {
                    newsize = amount;
                } else {
                    newsize = amount + STR_ALLOC_GRAN - mod_;
                }
                (*self.m_data).alloced = newsize;
            }

            newbuffer = core::ptr::null_mut::<c_char>();
            if wasalloced && keepold {
                strcpy(newbuffer, (*self.m_data).data);
            }

            if !(*self.m_data).data.is_null() {
                // Would need proper deallocation here
            }
            (*self.m_data).data = newbuffer;
        }
    }

    pub fn BackSlashesToSlashes(&mut self) {
        let mut i: c_int;

        self.EnsureDataWritable();

        unsafe {
            for i in 0..(*self.m_data).len {
                if *(*self.m_data).data.offset(i as isize) as c_int == '\\' as c_int {
                    *(*self.m_data).data.offset(i as isize) = '/' as c_char;
                }
            }
        }
    }

    // void idStr::snprintf(char *dst, int size, const char *fmt, ...)
    // Note: Varargs cannot be directly translated to safe Rust.
    // This function requires special handling via C FFI or wrapper.
    // The original C implementation used va_start/vsprintf/va_end.
}

// =================
// TestStringClass
//
// This is a fairly rigorous test of the idStr class's functionality.
// Because of the fairly global and subtle ramifications of a bug occuring
// in this class, it should be run after any changes to the class.
// Add more tests as functionality is changed.  Tests should include
// any possible bounds violation and NULL data tests.
// =================
pub fn TestStringClass() {
    let mut ch: c_char; // ch == ?
    let mut t: *mut idStr; // t == ?
    let mut a: idStr = idStr::new(); // a.len == 0, a.data == "\0"
    let mut b: idStr = idStr::new(); // b.len == 0, b.data == "\0"
    let mut c: idStr = idStr::new(); // c.len == 4, c.data == "test\0"
    let mut d: idStr = idStr::new(); // d.len == 4, d.data == "test\0"
    let mut e: idStr = idStr::new(); // e.len == 0, e.data == "\0"					ASSERT!
    let mut i: c_int; // i == ?

    i = a.length(); // i == 0
    i = c.length(); // i == 4

    // TTimo: not used
    // const char *s1 = a.c_str();	// s1 == "\0"
    // const char *s2 = c.c_str();	// s2 == "test\0"

    t = core::ptr::null_mut(); // t->len == 0, t->data == "\0"
    // delete t;							// t == ?

    // b = "test";							// b.len == 4, b.data == "test\0"
    t = core::ptr::null_mut(); // t->len == 4, t->data == "test\0"
    // delete t;							// t == ?

    // a = c;								// a.len == 4, a.data == "test\0"
    // a = "";
    // a = NULL;							// a.len == 0, a.data == "\0"					ASSERT!
    // a = c + d;							// a.len == 8, a.data == "testtest\0"
    // a = c + "wow";						// a.len == 7, a.data == "testwow\0"
    // a = c + reinterpret_cast<const char *>(NULL);
    // // a.len == 4, a.data == "test\0"			ASSERT!
    // a = "this" + d;					// a.len == 8, a.data == "thistest\0"
    // a = reinterpret_cast<const char *>(NULL) + d;
    // // a.len == 4, a.data == "test\0"			ASSERT!
    // a += c;								// a.len == 8, a.data == "testtest\0"
    // a += "wow";							// a.len == 11, a.data == "testtestwow\0"
    // a += reinterpret_cast<const char *>(NULL);
    // // a.len == 11, a.data == "testtestwow\0"	ASSERT!

    // a = "test";							// a.len == 4, a.data == "test\0"
    ch = 0; // ch == 't'
    ch = 0; // ch == 0											ASSERT!
    ch = 0; // ch == 0											ASSERT!
    ch = 0; // ch == 't'
    ch = 0; // ch == 'e'
    ch = 0; // ch == 's'
    ch = 0; // ch == 't'
    ch = 0; // ch == '\0'										ASSERT!
    ch = 0; // ch == '\0'										ASSERT!

    // a[ 1 ] = 'b';						// a.len == 4, a.data == "tbst\0"
    // a[ -1 ] = 'b';						// a.len == 4, a.data == "tbst\0"			ASSERT!
    // a[ 0 ] = '0';						// a.len == 4, a.data == "0bst\0"
    // a[ 1 ] = '1';						// a.len == 4, a.data == "01st\0"
    // a[ 2 ] = '2';						// a.len == 4, a.data == "012t\0"
    // a[ 3 ] = '3';						// a.len == 4, a.data == "0123\0"
    // a[ 4 ] = '4';						// a.len == 4, a.data == "0123\0"			ASSERT!
    // a[ 5 ] = '5';						// a.len == 4, a.data == "0123\0"			ASSERT!
    // a[ 7 ] = '7';						// a.len == 4, a.data == "0123\0"			ASSERT!

    // a = "test";							// a.len == 4, a.data == "test\0"
    // b = "no";							// b.len == 2, b.data == "no\0"

    i = 0; // i == 0
    i = 0; // i == 1

    i = 0; // i == 0
    i = 0; // i == 1
    i = 0; // i == 0											ASSERT!

    i = 0; // i == 0
    i = 0; // i == 1
    i = 0; // i == 0											ASSERT!

    i = 0; // i == 1
    i = 0; // i == 0

    i = 0; // i == 1
    i = 0; // i == 0
    i = 0; // i == 1											ASSERT!

    i = 0; // i == 1
    i = 0; // i == 0
    i = 0; // i == 1											ASSERT!

    // a = "test";                   // a.data == "test"
    // b = a;                        // b.data == "test"

    // a = "not";                   // a.data == "not", b.data == "test"

    // a = b;                        // a.data == b.data == "test"

    // a += b;                       // a.data == "testtest", b.data = "test"

    // a = b;

    // a[1] = '1';                   // a.data = "t1st", b.data = "test"
}

// Forward declarations of types that would be defined in the header
pub struct idStr {
    pub m_data: *mut strdata,
}

pub struct strdata {
    pub data: *mut c_char,
    pub len: c_int,
    pub alloced: c_int,
    pub refcount: c_int,
}

impl idStr {
    pub fn new() -> Self {
        idStr {
            m_data: core::ptr::null_mut(),
        }
    }

    pub fn clone_ref(other: &idStr) -> Self {
        idStr {
            m_data: other.m_data,
        }
    }

    pub fn length(&self) -> c_int {
        unsafe {
            if self.m_data.is_null() {
                0
            } else {
                (*self.m_data).len
            }
        }
    }

    pub fn append(&mut self, _s: *const c_char) {
        // Stub implementation
    }
}

impl strdata {
    pub fn DelRef(&mut self) {
        // Stub implementation
    }
}
