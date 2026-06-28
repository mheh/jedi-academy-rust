
/*****************************************************************************
 * name:		l_memory.h
 *
 * desc:		memory management
 *
 * $Archive: /source/code/botlib/l_memory.h $
 * $Author: Mrelusive $
 * $Revision: 2 $
 * $Modtime: 10/05/99 3:32p $
 * $Date: 10/05/99 3:42p $
 *
 *****************************************************************************/

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void, c_char, c_ulong};

//#define MEMDEBUG

#[cfg(feature = "memdebug")]
pub mod memdebug_fns {
    use core::ffi::{c_int, c_void, c_char, c_ulong};

    //allocate a memory block of the given size
    extern "C" {
        pub fn GetMemoryDebug(size: c_ulong, label: *const c_char, file: *const c_char, line: c_int) -> *mut c_void;
    }

    //allocate a memory block of the given size and clear it
    extern "C" {
        pub fn GetClearedMemoryDebug(size: c_ulong, label: *const c_char, file: *const c_char, line: c_int) -> *mut c_void;
    }

    //
    //allocate a memory block of the given size
    extern "C" {
        pub fn GetHunkMemoryDebug(size: c_ulong, label: *const c_char, file: *const c_char, line: c_int) -> *mut c_void;
    }

    //allocate a memory block of the given size and clear it
    extern "C" {
        pub fn GetClearedHunkMemoryDebug(size: c_ulong, label: *const c_char, file: *const c_char, line: c_int) -> *mut c_void;
    }
}

#[cfg(feature = "memdebug")]
#[macro_export]
macro_rules! GetMemory {
    ($size:expr) => {
        $crate::codemp::botlib::l_memory_h::memdebug_fns::GetMemoryDebug($size, concat!(stringify!($size)).as_ptr() as *const core::ffi::c_char, file!().as_ptr() as *const core::ffi::c_char, line!() as core::ffi::c_int)
    };
}

#[cfg(feature = "memdebug")]
#[macro_export]
macro_rules! GetClearedMemory {
    ($size:expr) => {
        $crate::codemp::botlib::l_memory_h::memdebug_fns::GetClearedMemoryDebug($size, concat!(stringify!($size)).as_ptr() as *const core::ffi::c_char, file!().as_ptr() as *const core::ffi::c_char, line!() as core::ffi::c_int)
    };
}

#[cfg(feature = "memdebug")]
#[macro_export]
macro_rules! GetHunkMemory {
    ($size:expr) => {
        $crate::codemp::botlib::l_memory_h::memdebug_fns::GetHunkMemoryDebug($size, concat!(stringify!($size)).as_ptr() as *const core::ffi::c_char, file!().as_ptr() as *const core::ffi::c_char, line!() as core::ffi::c_int)
    };
}

#[cfg(feature = "memdebug")]
#[macro_export]
macro_rules! GetClearedHunkMemory {
    ($size:expr) => {
        $crate::codemp::botlib::l_memory_h::memdebug_fns::GetClearedHunkMemoryDebug($size, concat!(stringify!($size)).as_ptr() as *const core::ffi::c_char, file!().as_ptr() as *const core::ffi::c_char, line!() as core::ffi::c_int)
    };
}

#[cfg(not(feature = "memdebug"))]
extern "C" {
    //allocate a memory block of the given size
    pub fn GetMemory(size: c_ulong) -> *mut c_void;

    //allocate a memory block of the given size and clear it
    pub fn GetClearedMemory(size: c_ulong) -> *mut c_void;
}

#[cfg(all(not(feature = "memdebug"), not(feature = "bspc")))]
extern "C" {
    //allocate a memory block of the given size
    pub fn GetHunkMemory(size: c_ulong) -> *mut c_void;

    //allocate a memory block of the given size and clear it
    pub fn GetClearedHunkMemory(size: c_ulong) -> *mut c_void;
}

#[cfg(all(not(feature = "memdebug"), feature = "bspc"))]
pub use GetMemory as GetHunkMemory;

#[cfg(all(not(feature = "memdebug"), feature = "bspc"))]
pub use GetClearedMemory as GetClearedHunkMemory;

//free the given memory block
extern "C" {
    pub fn FreeMemory(ptr: *mut c_void);
}

//returns the amount available memory
extern "C" {
    pub fn AvailableMemory() -> c_int;
}

//prints the total used memory size
extern "C" {
    pub fn PrintUsedMemorySize();
}

//print all memory blocks with label
extern "C" {
    pub fn PrintMemoryLabels();
}

//returns the size of the memory block in bytes
extern "C" {
    pub fn MemoryByteSize(ptr: *mut c_void) -> c_int;
}

//free all allocated memory
extern "C" {
    pub fn DumpMemory();
}
