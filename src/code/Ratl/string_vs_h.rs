////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// String
// ------
// Simple wrapper around a char[SIZE] array.
//
//
//
// NOTES:
//
//
//
////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::{c_char, c_int};

////////////////////////////////////////////////////////////////////////////////////////
// Includes
// Note: ratl_common.h functions declared via extern "C" below
////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////
// External C string functions (from ratl_common.h / libc)
////////////////////////////////////////////////////////////////////////////////////////
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strtok(s: *mut c_char, delim: *const c_char) -> *mut c_char;
    fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

////////////////////////////////////////////////////////////////////////////////////////
// Local str:: namespace stubs (from ratl_common.h)
////////////////////////////////////////////////////////////////////////////////////////
pub mod str {
    use super::*;

    pub fn len(s: *const c_char) -> usize {
        unsafe { strlen(s) }
    }

    pub fn ncpy(dest: *mut c_char, src: *const c_char, n: usize) {
        unsafe {
            strncpy(dest, src, n);
        }
    }

    pub fn cat(dest: *mut c_char, src: *const c_char) {
        unsafe {
            strcat(dest, src);
        }
    }

    pub fn tok(s: *mut c_char, delim: *const c_char) -> *mut c_char {
        unsafe { strtok(s, delim) }
    }

    pub fn cpy(dest: *mut c_char, src: *const c_char) {
        unsafe {
            let mut src_ptr = src;
            let mut dest_ptr = dest;
            loop {
                let c = *src_ptr;
                *dest_ptr = c;
                if c == 0 {
                    break;
                }
                src_ptr = src_ptr.add(1);
                dest_ptr = dest_ptr.add(1);
            }
        }
    }

    pub fn icmp(s1: *const c_char, s2: *const c_char) -> c_int {
        unsafe { stricmp(s1, s2) }
    }
}

pub mod ratl {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////////////////
    // The String Class
    ////////////////////////////////////////////////////////////////////////////////////////
    pub struct string_vs<const ARG_CAPACITY: usize> {
        // Capacity Enum
        // CAPACITY = ARG_CAPACITY

        // Data
        // In debug mode, includes 4 extra bytes for end marker; in release mode would be exactly CAPACITY
        // For faithful translation, we always include the 4 bytes but only fill/check them in debug mode
        mData: [c_char; ARG_CAPACITY + 4],
    }

    impl<const ARG_CAPACITY: usize> string_vs<ARG_CAPACITY> {
        pub const CAPACITY: usize = ARG_CAPACITY;

        fn FillTerminator(&mut self) {
            #[cfg(debug_assertions)]
            {
                self.mData[ARG_CAPACITY] = b'e' as c_char;
                self.mData[ARG_CAPACITY + 1] = b'n' as c_char;
                self.mData[ARG_CAPACITY + 2] = b'd' as c_char;
                self.mData[ARG_CAPACITY + 3] = 0;
            }
        }

        // Constructor
        pub fn new() -> Self {
            let mut s = string_vs {
                mData: [0; ARG_CAPACITY + 4],
            };
            s.mData[0] = 0;
            s.FillTerminator();
            s
        }

        // Debug destructor equivalent: check that end marker wasn't overwritten
        #[cfg(debug_assertions)]
        fn check_end_marker(&self) {
            //if you hit the below asserts, the end of the string was overwritten
            assert_eq!(self.mData[ARG_CAPACITY], b'e' as c_char);
            assert_eq!(self.mData[ARG_CAPACITY + 1], b'n' as c_char);
            assert_eq!(self.mData[ARG_CAPACITY + 2], b'd' as c_char);
            assert_eq!(self.mData[ARG_CAPACITY + 3], 0);
        }

        // Copy Constructor (from string_vs)
        pub fn from_string_vs(o: &string_vs<ARG_CAPACITY>) -> Self {
            unsafe {
                assert!(str::len(o.mData.as_ptr()) < ARG_CAPACITY);
            }
            let mut s = string_vs {
                mData: [0; ARG_CAPACITY + 4],
            };
            unsafe {
                str::ncpy(s.mData.as_mut_ptr(), o.mData.as_ptr(), ARG_CAPACITY);
                // Safe String Copy
            }
            s.mData[ARG_CAPACITY - 1] = 0; // Make Sure We Have A Null Terminated Str
            s.FillTerminator();
            s
        }

        // Copy Constructor (from const char*)
        pub fn from_c_str(s_ptr: *const c_char) -> Self {
            unsafe {
                assert!(str::len(s_ptr) < ARG_CAPACITY);
            }
            let mut s = string_vs {
                mData: [0; ARG_CAPACITY + 4],
            };
            unsafe {
                str::ncpy(s.mData.as_mut_ptr(), s_ptr, ARG_CAPACITY);
                // Safe String Copy
            }
            s.mData[ARG_CAPACITY - 1] = 0; // Make Sure We Have A Null Terminated Str
            s.FillTerminator();
            s
        }

        // Copy Assignment Operator
        pub fn assign(&mut self, s: *const c_char) -> &mut Self {
            unsafe {
                assert!(str::len(s) < ARG_CAPACITY);
                str::ncpy(self.mData.as_mut_ptr(), s, ARG_CAPACITY);
                // Safe String Copy
            }
            self.mData[ARG_CAPACITY - 1] = 0; // Make Sure We Have A Null Terminated Str
            self.FillTerminator();
            self
        }

        // Access To Raw Array
        pub fn c_str(&self) -> *const c_char {
            self.mData.as_ptr()
        }

        // Access To Raw Array (const version)
        pub fn c_str_const(&self) -> *const c_char {
            self.mData.as_ptr()
        }

        // Access To Raw Array (conversion operator)
        pub fn as_const_ptr(&self) -> *const c_char {
            self.mData.as_ptr()
        }

        // operator* (dereference)
        pub fn deref(&self) -> *const c_char {
            self.mData.as_ptr()
        }

        // How Many Characters Can This Hold
        pub fn capacity(&self) -> c_int {
            ARG_CAPACITY as c_int
        }

        // Length
        pub fn length(&self) -> c_int {
            unsafe {
                let len = str::len(self.mData.as_ptr());
                assert!(len < ARG_CAPACITY - 1);
                len as c_int
            }
        }

        // Character Bracket Operator
        pub fn index(&self, index: c_int) -> c_char {
            let idx = index as usize;
            assert!(idx < ARG_CAPACITY);
            self.mData[idx]
        }

        // Equality Operator
        pub fn operator_eq(&self, o: &string_vs<ARG_CAPACITY>) -> bool {
            unsafe {
                if stricmp(self.mData.as_ptr(), o.mData.as_ptr()) == 0 {
                    return true;
                }
            }
            false
        }

        // InEquality Operator
        pub fn operator_ne(&self, o: &string_vs<ARG_CAPACITY>) -> bool {
            unsafe {
                if str::icmp(self.mData.as_ptr(), o.mData.as_ptr()) != 0 {
                    return true;
                }
            }
            false
        }

        // Compare Less Than
        pub fn operator_lt(&self, o: &string_vs<ARG_CAPACITY>) -> bool {
            unsafe {
                if str::icmp(self.mData.as_ptr(), o.mData.as_ptr()) < 0 {
                    return true;
                }
            }
            false
        }

        // Compare Greater Than
        pub fn operator_gt(&self, o: &string_vs<ARG_CAPACITY>) -> bool {
            unsafe {
                if str::icmp(self.mData.as_ptr(), o.mData.as_ptr()) > 0 {
                    return true;
                }
            }
            false
        }

        // operator+=
        pub fn add_assign_string_vs(&mut self, o: &string_vs<ARG_CAPACITY>) {
            unsafe {
                if (str::len(self.mData.as_ptr()) + o.length() as usize) < ARG_CAPACITY {
                    // Only If It Is Safe
                    str::cat(self.mData.as_mut_ptr(), o.c_str());
                } else {
                    assert!(false, "string_vs overflow\n");
                }
            }
        }

        // operator+= (const char* version)
        pub fn add_assign_c_str(&mut self, s: *const c_char) {
            unsafe {
                if (str::len(self.mData.as_ptr()) + str::len(s)) < ARG_CAPACITY {
                    // Only If It Is Safe
                    str::cat(self.mData.as_mut_ptr(), s);
                } else {
                    assert!(false, "string_vs overflow\n");
                }
            }
        }

        // Get An Iterator To The First Token Seperated By Gap
        pub fn begin(&self, gap: *const c_char) -> tokenizer {
            tokenizer::new(self.mData.as_ptr(), gap)
        }

        // The Invalid Iterator, Use As A Stop Condition In Your For Loops
        pub fn end(&self) -> tokenizer {
            tokenizer::new_empty()
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Tokenizer
    //
    // The string tokenizer class is similar to an iterator in that it essentially
    // iterates over all the tokens in the string seperated by a common series of
    // delinating sequences.  For example:  " ,\t\n" would seperate tokens on spaces
    // commas, tabs and linefeeds.
    //
    // Iterating over string tokens is just like normal iteration:
    //
    // for (string_vs<CAPACITY>::tokenizer it=MyString.begin(" ,\t\n"); it!=MyString.end(); it++)
    // {
    //    const char* token = *it;
    // }
    //
    //
    // NOTE: This class is built upon the c library function strtok() which uses a
    // static working area, so having multiple tokenizers in multiple threads or just
    // plain at the same time is not safe.
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub struct tokenizer {
        mLoc: *mut c_char,
        mGap: [c_char; 15], // TOKEN_GAP_LEN = 15
    }

    impl tokenizer {
        // Constructors
        pub fn new(t: *const c_char, gap: *const c_char) -> Self {
            let mut tok = tokenizer {
                mLoc: std::ptr::null_mut(),
                mGap: [0; 15],
            };
            unsafe {
                str::ncpy(tok.mGap.as_mut_ptr(), gap, 15);
                // Safe String Copy
                tok.mGap[14] = 0; // Make Sure We Have A Null Terminated Str

                let mut temp = t as *mut c_char;
                tok.mLoc = str::tok(temp, tok.mGap.as_ptr());
            }
            tok
        }

        pub fn new_empty() -> Self {
            tokenizer {
                mLoc: std::ptr::null_mut(),
                mGap: [0; 15],
            }
        }

        // Assignment Operator
        pub fn assign(&mut self, t: &tokenizer) {
            self.mLoc = t.mLoc;
            unsafe {
                str::cpy(self.mGap.as_mut_ptr(), t.mGap.as_ptr());
            }
        }

        // Equality Operators
        pub fn operator_eq(&self, t: &tokenizer) -> bool {
            self.mLoc == t.mLoc
        }

        pub fn operator_ne(&self, t: &tokenizer) -> bool {
            !(self.operator_eq(t))
        }

        // DeReference Operator
        pub fn deref(&self) -> *const c_char {
            assert!(!self.mLoc.is_null());
            self.mLoc
        }

        // Inc & Dec Operators
        // Post-increment (operator++(int) in C++)
        pub fn inc(&mut self) {
            assert!(!self.mLoc.is_null() && self.mGap[0] != 0);
            self.mLoc = unsafe { str::tok(std::ptr::null_mut(), self.mGap.as_ptr()) };
        }
    }
}

// Type aliases
pub type TString_vs = ratl::string_vs<256>;
pub type TUIString_vs = ratl::string_vs<128>;
