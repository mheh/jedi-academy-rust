#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Stub for hfile from Rufl/hfile.h - needed for structural coherence
pub struct hfile;

// Forward declarations for C standard library functions called in this module
extern "C" {
    // From len() - helper function from ratl_common.h
    fn len(s: *const c_char) -> c_int;

    // From ctype.h
    fn toupper(c: c_int) -> c_int;
    fn tolower(c: c_int) -> c_int;

    // From stdio.h for varargs support
    fn sprintf(s: *mut c_char, format: *const c_char, ...) -> c_int;
}

pub mod ratl {
    use super::*;
    use core::ffi::{c_int, c_char, c_void};

    pub struct ratl_base;

    impl ratl_base {
        pub static mut OutputPrint: *const c_void = core::ptr::null();
    }

    #[cfg(not(target_os = "xbox"))]
    impl ratl_base {
        pub fn save(_file: &mut hfile) {
        }

        pub fn load(_file: &mut hfile) {
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // A Profile Print Function
    ////////////////////////////////////////////////////////////////////////////////////////
    #[cfg(not(feature = "final_build"))]
    impl ratl_base {
        pub unsafe fn ProfilePrint(format: *const c_char) {
            static mut string: [[c_char; 1024]; 2] = [[0; 1024]; 2];	// in case this is called by nested functions
            static mut index: c_int = 0;
            static mut nFormat: [c_char; 300] = [0; 300];

            // Tack On The Standard Format Around The Given Format
            //-----------------------------------------------------
            sprintf(
                nFormat.as_mut_ptr(),
                b"[PROFILE] %s\n\0".as_ptr() as *const c_char,
                format,
            );

            // Resolve Remaining Elipsis Parameters Into Newly Formated String
            //-----------------------------------------------------------------
            let buf = &mut string[(index & 1) as usize];
            index += 1;

            // Note: Rust does not have native support for C varargs (va_list/va_start/va_end).
            // The original C code would use:
            //   va_list argptr;
            //   va_start (argptr, format);
            //   vsprintf (buf, nFormat, argptr);
            //   va_end (argptr);
            // This would require either FFI to a C wrapper or platform-specific code.
            // For now, the formatted string is prepared above via sprintf.

            // Print It To Debug Output Console
            //----------------------------------
            if !OutputPrint.is_null() {
                let OutputPrintFcn: unsafe extern "C" fn(*const c_char) =
                    core::mem::transmute(OutputPrint);
                OutputPrintFcn(buf.as_ptr());
            }
        }
    }

    #[cfg(feature = "final_build")]
    impl ratl_base {
        pub fn ProfilePrint(_format: *const c_char) {
        }
    }

    #[cfg(debug_assertions)]
    pub mod debug_statics {
        use core::ffi::c_int;

        pub static mut HandleSaltValue: c_int = 1027; //this is used in debug for global uniqueness of handles
        pub static mut FoolTheOptimizer: c_int = 5;		//this is used to make sure certain things aren't optimized out
    }

    pub mod str {
        use super::super::*;
        use core::ffi::c_char;

        pub fn to_upper(dest: *mut c_char) {
            unsafe {
                let length = len(dest as *const c_char);
                for i in 0..length {
                    let ch = *dest.offset(i as isize);
                    *dest.offset(i as isize) = (toupper(ch as c_int) as c_char);
                }
            }
        }

        pub fn to_lower(dest: *mut c_char) {
            unsafe {
                let length = len(dest as *const c_char);
                for i in 0..length {
                    let ch = *dest.offset(i as isize);
                    *dest.offset(i as isize) = (tolower(ch as c_int) as c_char);
                }
            }
        }

        pub fn printf(dest: *mut c_char, formatS: *const c_char) {
            unsafe {
                // Note: Original C code signature had varargs:
                //   void printf(char *dest, const char *formatS, ...)
                // The original implementation used:
                //   va_list argptr;
                //   va_start (argptr, formatS);
                //   vsprintf (dest, formatS, argptr);
                //   va_end (argptr);
                // Rust does not provide native support for C-style varargs in user code.
                // Callers would need to use FFI or C wrappers for actual varargs support.
            }
        }
    }
}
