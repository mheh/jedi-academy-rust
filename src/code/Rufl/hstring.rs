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

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use core::ffi::{c_int, c_char};
use core::ptr;

////////////////////////////////////////////////////////////////////////////////////////
// Includes (external libc functions)
////////////////////////////////////////////////////////////////////////////////////////
extern "C" {
    fn strlen(s: *const c_char) -> usize;
}

////////////////////////////////////////////////////////////////////////////////////////
// Defines
////////////////////////////////////////////////////////////////////////////////////////
const MAX_HASH: usize = 16384;           // How Many Hash
const BLOCK_SIZE: usize = 65536;         // Size of a string storage block in bytes.

// C void type alias for clarity in FFI code
#[allow(non_camel_case_types)]
type c_void = core::ffi::c_void;

////////////////////////////////////////////////////////////////////////////////////////
// Local stub for ratl::hash_pool template
// Faithful representation of: template <int SIZE, int SIZE_HANDLES> class hash_pool
////////////////////////////////////////////////////////////////////////////////////////
#[repr(C)]
struct TStrPool {
    mHandles: [c_int; MAX_HASH],         // each handle holds the start index of it's data
    mDataAlloc: c_int,                   // where the next chunck of data will go
    mData: [c_char; BLOCK_SIZE],

    #[cfg(debug_assertions)]
    mFinds: c_int,                       // counts how many total finds have run
    #[cfg(debug_assertions)]
    mCurrentCollisions: c_int,           // counts how many collisions on the last find
    #[cfg(debug_assertions)]
    mTotalCollisions: c_int,             // counts the total number of collisions
    #[cfg(debug_assertions)]
    mTotalAllocs: c_int,
}

impl TStrPool {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constructor (clear)
    ////////////////////////////////////////////////////////////////////////////////////
    fn new() -> Self {
        TStrPool {
            mHandles: [0; MAX_HASH],
            mDataAlloc: 1,
            mData: [0; BLOCK_SIZE],
            #[cfg(debug_assertions)]
            mFinds: 0,
            #[cfg(debug_assertions)]
            mCurrentCollisions: 0,
            #[cfg(debug_assertions)]
            mTotalCollisions: 0,
            #[cfg(debug_assertions)]
            mTotalAllocs: 0,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // This function searches for a handle which already stores the data (assuming the
    // handle is a hash within range SIZE_HANDLES).
    //
    // If it failes, it returns false, and the handle passed in points to the next
    // free slot.
    ////////////////////////////////////////////////////////////////////////////////////
    fn find_existing(&mut self, handle: &mut c_int, data: *const c_void, datasize: c_int) -> bool {
        #[cfg(debug_assertions)]
        {
            self.mFinds += 1;
            self.mCurrentCollisions = 0;
        }

        while self.mHandles[*handle as usize] != 0 {
            // So long as a handle exists there
            if self.mem_eql(
                ptr::addr_of_mut!(self.mData[self.mHandles[*handle as usize] as usize]) as *mut c_void,
                data as *mut c_void,
                datasize,
            ) {
                return true; // found
            }
            *handle = (*handle + 1) & ((MAX_HASH as c_int) - 1); // incriment the handle

            #[cfg(debug_assertions)]
            {
                self.mCurrentCollisions += 1;
                self.mTotalCollisions += 1;

                //assert(mCurrentCollisions < 16);		// If We Had 16+ Collisions, Hash May Be Inefficient.
                //                                         // Evaluate SIZE and SIZEHANDLES
            }
        }
        false // failed to find
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // A simple hash function for the range of [0, SIZE_HANDLES]
    ////////////////////////////////////////////////////////////////////////////////////
    fn hash(&self, data: *const c_void, datasize: c_int) -> c_int {
        let mut h: c_int = 0;
        for i in 0..datasize {
            let byte_val = unsafe { *(data as *const c_char).add(i as usize) } as c_int;
            h += byte_val * (i + 119); // 119.  Prime Number?
        }
        h &= (MAX_HASH as c_int) - 1; // zero out bits beyoned SIZE_HANDLES
        h
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // The Number Of Bytes Allocated
    ////////////////////////////////////////////////////////////////////////////////////
    fn size(&self) -> c_int {
        self.mDataAlloc
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // This is the primary functionality of the hash pool.  It will search for existing
    // data of the same size, and failing to find any, it will append the data to the
    // memory.
    //
    // In both cases, it gives you a handle to look up the data later.
    ////////////////////////////////////////////////////////////////////////////////////
    fn get_handle(&mut self, data: *const c_void, datasize: c_int) -> c_int {
        let mut handle = self.hash(data, datasize); // Initialize Our Handle By Hash Fcn
        if !self.find_existing(&mut handle, data, datasize) {
            assert!(
                (self.mDataAlloc as usize) + (datasize as usize) < BLOCK_SIZE,
                "Is There Enough Memory?"
            );

            #[cfg(debug_assertions)]
            {
                self.mTotalAllocs += 1;
            }

            self.mem_cpy(
                ptr::addr_of_mut!(self.mData[self.mDataAlloc as usize]) as *mut c_void,
                data as *mut c_void,
                datasize,
            ); // Copy Data To Memory
            self.mHandles[handle as usize] = self.mDataAlloc; // Mark Memory In Hash Tbl
            self.mDataAlloc += datasize; // Adjust Next Alloc Location
        }
        handle // Return The Hash Tbl handleess
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Constant Access Operator
    ////////////////////////////////////////////////////////////////////////////////////
    fn index(&self, handle: c_int) -> *const c_void {
        assert!(handle >= 0 && handle < (MAX_HASH as c_int));
        unsafe { ptr::addr_of!(self.mData[self.mHandles[handle as usize] as usize]) as *const c_void }
    }

    #[cfg(debug_assertions)]
    fn average_collisions(&self) -> f32 {
        if self.mFinds == 0 {
            0.0
        } else {
            (self.mTotalCollisions as f32) / (self.mFinds as f32)
        }
    }

    #[cfg(debug_assertions)]
    fn total_allocs(&self) -> c_int {
        self.mTotalAllocs
    }

    #[cfg(debug_assertions)]
    fn total_finds(&self) -> c_int {
        self.mFinds
    }

    #[cfg(debug_assertions)]
    fn total_collisions(&self) -> c_int {
        self.mTotalCollisions
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Helper function - memory equality check
    ////////////////////////////////////////////////////////////////////////////////////
    fn mem_eql(&self, a: *mut c_void, b: *mut c_void, size: c_int) -> bool {
        for i in 0..(size as usize) {
            unsafe {
                if *(a as *mut u8).add(i) != *(b as *mut u8).add(i) {
                    return false;
                }
            }
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Helper function - memory copy
    ////////////////////////////////////////////////////////////////////////////////////
    fn mem_cpy(&mut self, dst: *mut c_void, src: *mut c_void, size: c_int) {
        for i in 0..(size as usize) {
            unsafe {
                *(dst as *mut u8).add(i) = *(src as *mut u8).add(i);
            }
        }
    }
}

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

////////////////////////////////////////////////////////////////////////////////////////
// The Hash Pool
////////////////////////////////////////////////////////////////////////////////////////
fn Pool() -> &'static mut TStrPool {
    static mut TSP: TStrPool = TStrPool {
        mHandles: [0; MAX_HASH],
        mDataAlloc: 1,
        mData: [0; BLOCK_SIZE],
        #[cfg(debug_assertions)]
        mFinds: 0,
        #[cfg(debug_assertions)]
        mCurrentCollisions: 0,
        #[cfg(debug_assertions)]
        mTotalCollisions: 0,
        #[cfg(debug_assertions)]
        mTotalAllocs: 0,
    };
    unsafe { &mut TSP }
}

#[cfg(not(target_os = "xbox"))]
impl hstring {
    ////////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn new() -> Self {
        hstring {
            mHandle: 0,
            #[cfg(debug_assertions)]
            mStr: ptr::null_mut(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn from_cstr(str: *const c_char) -> Self {
        let mut h = hstring {
            mHandle: 0,
            #[cfg(debug_assertions)]
            mStr: ptr::null_mut(),
        };
        h.init(str);
        h
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn from_hstring(str_: &hstring) -> Self {
        hstring {
            mHandle: str_.mHandle,

            #[cfg(debug_assertions)]
            mStr: str_.mStr,
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Assignment
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn assign_cstr(&mut self, str: *const c_char) -> &mut Self {
        self.init(str);
        self
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Assignment
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn assign_hstring(&mut self, str_: &hstring) -> &mut Self {
        self.mHandle = str_.mHandle;

        #[cfg(debug_assertions)]
        {
            self.mStr = str_.mStr;
        }
        self
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Comparison
    ////////////////////////////////////////////////////////////////////////////////////////
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

    ////////////////////////////////////////////////////////////////////////////////////////
    // Conversion
    ////////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn c_str(&self) -> *const c_char {
        if self.mHandle == 0 {
            return b"\0".as_ptr() as *const c_char;
        }
        Pool().index(self.mHandle) as *const c_char
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn operator_ptr(&self) -> *const c_char {
        // const char*	operator *(void) const;
        self.c_str()
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn length(&self) -> c_int {
        unsafe { strlen(self.c_str()) as c_int }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn handle(&self) -> c_int {
        self.mHandle
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Access Functions
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        // bool		empty()	const							{return handle()==0;}
        self.handle() == 0
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Helper Functions
    ////////////////////////////////////////////////////////////////////////////////////
    ////////////////////////////////////////////////////////////////////////////////////
    //
    ////////////////////////////////////////////////////////////////////////////////////
    fn init(&mut self, str: *const c_char) {
        if str.is_null() {
            self.mHandle = 0;
        } else {
            let len = unsafe { strlen(str) as c_int } + 1; // +1 for null character
            self.mHandle = Pool().get_handle(str as *const c_void, len);
        }
        #[cfg(debug_assertions)]
        {
            self.mStr = Pool().index(self.mHandle) as *mut c_char;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Debug Statistics Routines
    ////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub fn ave_collisions() -> f32 {
        Pool().average_collisions()
    }

    #[cfg(debug_assertions)]
    pub fn total_strings() -> c_int {
        Pool().total_allocs()
    }

    #[cfg(debug_assertions)]
    pub fn total_bytes() -> c_int {
        Pool().size()
    }

    #[cfg(debug_assertions)]
    pub fn total_finds() -> c_int {
        Pool().total_finds()
    }

    #[cfg(debug_assertions)]
    pub fn total_collisions() -> c_int {
        Pool().total_collisions()
    }
}

#[cfg(target_os = "xbox")]
pub mod dllNamespace {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////////////////
    // Constructor
    ////////////////////////////////////////////////////////////////////////////////////////
    impl hstring {
        pub fn new() -> Self {
            hstring {
                mHandle: 0,
                #[cfg(debug_assertions)]
                mStr: ptr::null_mut(),
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Constructor
        ////////////////////////////////////////////////////////////////////////////////////////
        pub fn from_cstr(str: *const c_char) -> Self {
            let mut h = hstring {
                mHandle: 0,
                #[cfg(debug_assertions)]
                mStr: ptr::null_mut(),
            };
            h.init(str);
            h
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Constructor
        ////////////////////////////////////////////////////////////////////////////////////////
        pub fn from_hstring(str_: &hstring) -> Self {
            hstring {
                mHandle: str_.mHandle,

                #[cfg(debug_assertions)]
                mStr: str_.mStr,
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Assignment
        ////////////////////////////////////////////////////////////////////////////////////////
        pub fn assign_cstr(&mut self, str: *const c_char) -> &mut Self {
            self.init(str);
            self
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Assignment
        ////////////////////////////////////////////////////////////////////////////////////////
        pub fn assign_hstring(&mut self, str_: &hstring) -> &mut Self {
            self.mHandle = str_.mHandle;

            #[cfg(debug_assertions)]
            {
                self.mStr = str_.mStr;
            }
            self
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        // Comparison
        ////////////////////////////////////////////////////////////////////////////////////////
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

        ////////////////////////////////////////////////////////////////////////////////////////
        // Conversion
        ////////////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////////////
        //
        ////////////////////////////////////////////////////////////////////////////////////////
        pub fn c_str(&self) -> *const c_char {
            if self.mHandle == 0 {
                return b"\0".as_ptr() as *const c_char;
            }
            Pool().index(self.mHandle) as *const c_char
        }

        ////////////////////////////////////////////////////////////////////////////////////////
        //
        ////////////////////////////////////////////////////////////////////////////////////////
        pub fn operator_ptr(&self) -> *const c_char {
            // const char*	operator *(void) const;
            self.c_str()
        }

        ////////////////////////////////////////////////////////////////////////////////////
        //
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn length(&self) -> c_int {
            unsafe { strlen(self.c_str()) as c_int }
        }

        ////////////////////////////////////////////////////////////////////////////////////
        //
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn handle(&self) -> c_int {
            self.mHandle
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Access Functions
        ////////////////////////////////////////////////////////////////////////////////////
        pub fn empty(&self) -> bool {
            // bool		empty()	const							{return handle()==0;}
            self.handle() == 0
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Helper Functions
        ////////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////////
        //
        ////////////////////////////////////////////////////////////////////////////////////
        fn init(&mut self, str: *const c_char) {
            if str.is_null() {
                self.mHandle = 0;
            } else {
                let len = unsafe { strlen(str) as c_int } + 1; // +1 for null character
                self.mHandle = Pool().get_handle(str as *const c_void, len);
            }
            #[cfg(debug_assertions)]
            {
                self.mStr = Pool().index(self.mHandle) as *mut c_char;
            }
        }

        ////////////////////////////////////////////////////////////////////////////////////
        // Debug Statistics Routines
        ////////////////////////////////////////////////////////////////////////////////////
        #[cfg(debug_assertions)]
        pub fn ave_collisions() -> f32 {
            Pool().average_collisions()
        }

        #[cfg(debug_assertions)]
        pub fn total_strings() -> c_int {
            Pool().total_allocs()
        }

        #[cfg(debug_assertions)]
        pub fn total_bytes() -> c_int {
            Pool().size()
        }

        #[cfg(debug_assertions)]
        pub fn total_finds() -> c_int {
            Pool().total_finds()
        }

        #[cfg(debug_assertions)]
        pub fn total_collisions() -> c_int {
            Pool().total_collisions()
        }
    }
}
