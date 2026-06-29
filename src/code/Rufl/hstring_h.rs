////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD USEFUL FUNCTION LIBRARY
//  (c) 2002 Activision
//
//
// Handle String
// -------------
// Handle strings are allocated once in a static buffer (with a hash index), and are
// never cleared out.  You should use these for very common string names which are
// redundant or intended to last a long time.
//
// Handle strings are also good for comparison and storage because they compare only
// the handles, which are simple unique integers.
//
////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::{c_int, c_char};

////////////////////////////////////////////////////////////////////////////////////////
// The Handle String Class
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
pub struct hstring {
    ////////////////////////////////////////////////////////////////////////////////////
    // Data
    ////////////////////////////////////////////////////////////////////////////////////
    mHandle: c_int,

    #[cfg(debug_assertions)]
    mStr: *mut c_char,
}

impl hstring {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        todo!()
    }

    pub fn from_cstr(str_: *const c_char) -> Self {
        todo!()
    }

    pub fn clone_from_hstring(str_: &hstring) -> Self {
        todo!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Assignment
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn assign_cstr(&mut self, str_: *const c_char) -> &mut Self {
        todo!()
    }

    pub fn assign_hstring(&mut self, str_: &hstring) -> &mut Self {
        todo!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Comparison
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn eq(&self, str_: &hstring) -> bool {
        // bool		operator== (const hstring &str) const	{return (mHandle==str.mHandle);}
        self.mHandle == str_.mHandle
    }

    pub fn lt(&self, str_: &hstring) -> bool {
        // bool		operator<  (const hstring &str) const	{return (mHandle< str.mHandle);}
        self.mHandle < str_.mHandle
    }

    pub fn not(&self) -> bool {
        // bool		operator!  () const						{return (mHandle==0);}
        self.mHandle == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Conversion
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn c_str(&self) -> *const c_char {
        todo!()
    }

    pub fn as_ptr_operator(&self) -> *const c_char {
        // const char*	operator *(void) const;
        todo!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access Functions
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn length(&self) -> c_int {
        todo!()
    }

    pub fn handle(&self) -> c_int {
        self.mHandle
    }

    pub fn empty(&self) -> bool {
        // bool		empty()	const							{return handle()==0;}
        self.handle() == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Debug Statistics Routines
    ////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub fn ave_collisions() -> f32 {
        todo!()
    }

    #[cfg(debug_assertions)]
    pub fn total_strings() -> c_int {
        todo!()
    }

    #[cfg(debug_assertions)]
    pub fn total_bytes() -> c_int {
        todo!()
    }

    #[cfg(debug_assertions)]
    pub fn total_finds() -> c_int {
        todo!()
    }

    #[cfg(debug_assertions)]
    pub fn total_collisions() -> c_int {
        todo!()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Helper Functions
    ////////////////////////////////////////////////////////////////////////////////////
    fn init(&mut self, str_: *const c_char) {
        todo!()
    }
}
