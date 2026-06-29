// ////////////////////////////////////////////////////////////////////////////////////////
// // RAVEN STANDARD TEMPLATE LIBRARY
// //  (c) 2002 Activision
// //
// //
// // Common
// // ------
// // The raven libraries contain a number of common defines, enums, and typedefs which
// // need to be accessed by all templates.  Each of these is included here.
// //
// // Also included is a safeguarded assert file for all the asserts in RTL.
// //
// // This file is included in EVERY TEMPLATE, so it should be very light in order to
// // reduce compile times.
// //
// //
// // Format
// // ------
// // In order to simplify code and provide readability, the template library has some
// // standard formats.  Any new templates or functions should adhere to these formats:
// //
// // - All memory is statically allocated, usually by parameter SIZE
// // - All classes provide an enum which defines constant variables, including CAPACITY
// // - All classes which moniter the number of items allocated provide the following functions:
// //     size()   - the number of objects
// //     empty()  - does the container have zero objects
// //     full()   - does the container have any room left for more objects
// //     clear()  - remove all objects
// //
// //
// // - Functions are defined in the following order:
// //     Capacity
// //     Constructors  (copy, from string, etc...)
// //     Range		 (size(), empty(), full(), clear(), etc...)
// //     Access        (operator[], front(), back(), etc...)
// //     Modification  (add(), remove(), push(), pop(), etc...)
// //     Iteration     (begin(), end(), insert(), erase(), find(), etc...)
// //
// //
// // NOTES:
// //
// //
// //
// ////////////////////////////////////////////////////////////////////////////////////////

use core::ffi::{c_int, c_char, c_void};
use core::mem;
use std::ffi::CStr;

// ////////////////////////////////////////////////////////////////////////////////////////
// // Forward Dec.
// ////////////////////////////////////////////////////////////////////////////////////////
// pub struct hfile; // Forward declaration for file I/O (not implemented in this stub)

// // I don't know why this needs to be in the global namespace, but it does
// pub struct TRatlNew;
//
// #[allow(non_snake_case)]
// pub unsafe fn operator_new(size: usize, where_: *mut TRatlNew) -> *mut c_void {
//     where_ as *mut c_void
// }
//
// #[allow(non_snake_case)]
// pub unsafe fn operator_delete(_p: *mut c_void, _where: *mut TRatlNew) {
//     // No-op
// }

pub mod ratl {
    use core::ffi::c_void;
    use std::ffi::CStr;
    use std::ptr;

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // Debug globals
    // ////////////////////////////////////////////////////////////////////////////////////////
    #[cfg(debug_assertions)]
    pub static mut HandleSaltValue: i32 = 0; // this is used in debug for global uniqueness of handles
    #[cfg(debug_assertions)]
    pub static mut FoolTheOptimizer: i32 = 0; // this is used to make sure certain things aren't optimized out

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // All Raven Template Library Internal Memory Operations
    // //
    // // This is mostly for future use.  For now, they only provide a simple interface with
    // // a couple extra functions (eql and clr).
    // ////////////////////////////////////////////////////////////////////////////////////////
    pub mod mem {
        use core::ffi::c_void;
        use std::ptr;

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // The Align Struct Is The Root Memory Structure for Inheritance and Object Semantics
        // //
        // // In most cases, we just want a simple int.  However, sometimes we need to use an
        // // unsigned character array
        // //
        // ////////////////////////////////////////////////////////////////////////////////////////
        #[repr(C)]
        #[cfg(target_env = "msvc")]
        pub struct alignStruct {
            space: i32,
        }

        #[repr(C, align(16))]
        #[cfg(not(target_env = "msvc"))]
        pub struct alignStruct {
            space: [u8; 16],
        }

        #[inline]
        pub unsafe fn cpy(dest: *mut c_void, src: *const c_void, count: usize) -> *mut c_void {
            ptr::copy_nonoverlapping(src, dest, count);
            dest
        }

        #[inline]
        pub unsafe fn set(dest: *mut c_void, c: i32, count: usize) -> *mut c_void {
            ptr::write_bytes(dest as *mut u8, c as u8, count);
            dest
        }

        #[inline]
        pub unsafe fn cmp(buf1: *const c_void, buf2: *const c_void, count: usize) -> i32 {
            let slice1 = std::slice::from_raw_parts(buf1 as *const u8, count);
            let slice2 = std::slice::from_raw_parts(buf2 as *const u8, count);
            slice1.cmp(slice2) as i32
        }

        #[inline]
        pub unsafe fn eql(buf1: *const c_void, buf2: *const c_void, count: usize) -> bool {
            cmp(buf1, buf2, count) == 0
        }

        #[inline]
        pub unsafe fn zero(dest: *mut c_void, count: usize) -> *mut c_void {
            set(dest, 0, count)
        }

        #[inline]
        pub unsafe fn cpy_typed<T>(dest: *mut T, src: *const T) {
            cpy(
                dest as *mut c_void,
                src as *const c_void,
                std::mem::size_of::<T>(),
            );
        }

        #[inline]
        pub unsafe fn set_typed<T>(dest: *mut T, c: i32) {
            set(
                dest as *mut c_void,
                c,
                std::mem::size_of::<T>(),
            );
        }

        #[inline]
        pub unsafe fn swap<T>(s1: *mut T, s2: *mut T) {
            let mut temp: [u8; std::mem::size_of::<T>()] =
                [0; std::mem::size_of::<T>()];
            cpy_typed(temp.as_mut_ptr() as *mut T, s1);
            cpy_typed(s1, s2);
            cpy_typed(s2, temp.as_ptr() as *const T);
        }

        #[inline]
        pub unsafe fn cmp_typed<T>(buf1: *const T, buf2: *const T) -> i32 {
            cmp(
                buf1 as *const c_void,
                buf2 as *const c_void,
                std::mem::size_of::<T>(),
            )
        }

        #[inline]
        pub unsafe fn eql_typed<T>(buf1: *const T, buf2: *const T) -> bool {
            cmp_typed(buf1, buf2) == 0
        }

        #[inline]
        pub unsafe fn zero_typed<T>(dest: *mut T) {
            set(
                dest as *mut c_void,
                0,
                std::mem::size_of::<T>(),
            );
        }
    }

    pub mod str {
        use std::ffi::CStr;
        use std::os::raw::c_char;

        #[inline]
        pub unsafe fn len(src: *const c_char) -> i32 {
            CStr::from_ptr(src).len() as i32
        }

        #[inline]
        pub unsafe fn cpy(dest: *mut c_char, src: *const c_char) {
            let src_str = CStr::from_ptr(src);
            let bytes = src_str.to_bytes_with_nul();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest as *mut u8, bytes.len());
        }

        #[inline]
        pub unsafe fn ncpy(dest: *mut c_char, src: *const c_char, destBufferLen: i32) {
            let src_str = CStr::from_ptr(src);
            let bytes = src_str.to_bytes_with_nul();
            let copy_len = (bytes.len()).min(destBufferLen as usize);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest as *mut u8, copy_len);
            if copy_len < destBufferLen as usize {
                *(dest.add(copy_len - 1)) = 0;
            }
        }

        #[inline]
        pub unsafe fn cat(dest: *mut c_char, src: *const c_char) {
            let dest_len = len(dest) as usize;
            cpy(dest.add(dest_len), src);
        }

        #[inline]
        pub unsafe fn ncat(dest: *mut c_char, src: *const c_char, destBufferLen: i32) {
            let dest_len = len(dest) as usize;
            ncpy(
                dest.add(dest_len),
                src,
                destBufferLen - dest_len as i32,
            );
        }

        #[inline]
        pub unsafe fn cmp(s1: *const c_char, s2: *const c_char) -> i32 {
            let str1 = CStr::from_ptr(s1);
            let str2 = CStr::from_ptr(s2);
            str1.cmp(str2) as i32
        }

        #[inline]
        pub unsafe fn eql(s1: *const c_char, s2: *const c_char) -> bool {
            cmp(s1, s2) == 0
        }

        #[inline]
        pub unsafe fn icmp(s1: *const c_char, s2: *const c_char) -> i32 {
            let str1 = CStr::from_ptr(s1).to_string_lossy();
            let str2 = CStr::from_ptr(s2).to_string_lossy();
            str1.to_lowercase().cmp(&str2.to_lowercase()) as i32
        }

        #[inline]
        pub unsafe fn cmpi(s1: *const c_char, s2: *const c_char) -> i32 {
            icmp(s1, s2)
        }

        #[inline]
        pub unsafe fn ieql(s1: *const c_char, s2: *const c_char) -> bool {
            icmp(s1, s2) == 0
        }

        #[inline]
        pub unsafe fn eqli(s1: *const c_char, s2: *const c_char) -> bool {
            ieql(s1, s2)
        }

        #[inline]
        pub unsafe fn tok(s: *mut c_char, gap: *const c_char) -> *mut c_char {
            // This is a simplified stub; real implementation would use C's strtok
            std::ptr::null_mut()
        }

        pub unsafe fn to_upper(dest: *mut c_char);
        pub unsafe fn to_lower(dest: *mut c_char);
        pub unsafe fn printf(dest: *mut c_char, formatS: *const c_char, ...);
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // The Raven Template Library Compile Assert
    // //
    // // If, during compile time the stuff under (condition) is zero, this code will not
    // // compile.
    // ////////////////////////////////////////////////////////////////////////////////////////
    #[repr(transparent)]
    pub struct compile_assert<const CONDITION: bool>;

    impl<const CONDITION: bool> compile_assert<CONDITION> {
        #[inline]
        pub fn new() -> Self {
            compile_assert
        }

        #[inline]
        pub fn call(&self) -> i32 {
            1
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // The Raven Template Library Base Class
    // //
    // // This is the base class for all the Raven Template Library container classes like
    // // vector_vs and pool_vs.
    // //
    // // This class might be a good place to put memory profile code in the future.
    // //
    // ////////////////////////////////////////////////////////////////////////////////////////
    pub struct ratl_base {
        pub OutputPrint: *mut c_void,
    }

    impl ratl_base {
        #[cfg(not(target_os = "xbox"))]
        pub fn save(&mut self, file: &mut std::any::Any) {
            // Placeholder
        }

        #[cfg(not(target_os = "xbox"))]
        pub fn load(&mut self, file: &mut std::any::Any) {
            // Placeholder
        }

        pub fn ProfilePrint(&self, format: *const std::os::raw::c_char) {
            // Placeholder
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // this is a simplified version of bits_vs
    // ////////////////////////////////////////////////////////////////////////////////////////
    pub struct bits_base<const SZ: usize> {
        mV: [u32; Self::ARRAY_SIZE],
    }

    impl<const SZ: usize> bits_base<SZ> {
        const BITS_SHIFT: usize = 5;
        const BITS_INT_SIZE: usize = 32;
        const BITS_AND: usize = (Self::BITS_INT_SIZE - 1);
        const ARRAY_SIZE: usize = ((SZ + Self::BITS_AND) / (Self::BITS_INT_SIZE));
        const BYTE_SIZE: usize = (Self::ARRAY_SIZE * std::mem::size_of::<u32>());

        const SIZE: usize = SZ;
        const CAPACITY: usize = SZ;

        pub const fn new(init: bool, initValue: bool) -> Self {
            bits_base {
                mV: [if initValue { 0xff } else { 0 }; Self::ARRAY_SIZE],
            }
        }

        pub fn clear(&mut self) {
            for i in 0..Self::ARRAY_SIZE {
                self.mV[i] = 0;
            }
        }

        pub fn set(&mut self) {
            for i in 0..Self::ARRAY_SIZE {
                self.mV[i] = 0xff;
            }
        }

        pub fn set_bit(&mut self, i: usize) {
            debug_assert!(i >= 0 && i < Self::SIZE);
            self.mV[i >> Self::BITS_SHIFT] |= (1 << (i & Self::BITS_AND));
        }

        pub fn clear_bit(&mut self, i: usize) {
            debug_assert!(i >= 0 && i < Self::SIZE);
            self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
        }

        pub fn mark_bit(&mut self, i: usize, set: bool) {
            debug_assert!(i >= 0 && i < Self::SIZE);
            if set {
                self.mV[i >> Self::BITS_SHIFT] |= (1 << (i & Self::BITS_AND));
            } else {
                self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
            }
        }

        pub fn at(&self, i: usize) -> bool {
            debug_assert!(i >= 0 && i < Self::SIZE);
            (self.mV[i >> Self::BITS_SHIFT] & (1 << (i & Self::BITS_AND))) != 0
        }

        pub fn next_bit(&self, start: usize, onBit: bool) -> usize {
            debug_assert!(start >= 0 && start <= Self::SIZE);
            if start >= Self::SIZE {
                return Self::SIZE;
            }

            // Get The Word Which Contains The Start Bit & Mask Out Everything Before The Start Bit
            //--------------------------------------------------------------------------------------
            let mut v = self.mV[start >> Self::BITS_SHIFT];
            if !onBit {
                v = !v;
            }
            v >>= (start & 31);

            let mut idx = start;

            // Search For The First Non Zero Word In The Array
            //-------------------------------------------------
            while v == 0 {
                idx = (idx & !(Self::BITS_INT_SIZE - 1)) + Self::BITS_INT_SIZE;
                if idx >= Self::SIZE {
                    return Self::SIZE;
                }
                v = self.mV[idx >> Self::BITS_SHIFT];
                if !onBit {
                    v = !v;
                }
            }

            // So, We've Found A Non Zero Word, So Start Masking Against Parts To Skip Over Bits
            //-----------------------------------------------------------------------------------
            if (v & 0xffff) == 0 {
                idx += 16;
                v >>= 16;
            }
            if (v & 0xff) == 0 {
                idx += 8;
                v >>= 8;
            }
            if (v & 0xf) == 0 {
                idx += 4;
                v >>= 4;
            }

            // Time To Search Each Bit
            //-------------------------
            while (v & 1) == 0 {
                idx += 1;
                v >>= 1;
            }
            if idx >= Self::SIZE {
                return Self::SIZE;
            }
            idx
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // Raven Standard Compare Class
    // ////////////////////////////////////////////////////////////////////////////////////////
    #[repr(C)]
    pub struct ratl_compare {
        pub mCost: f32,
        pub mHandle: i32,
    }

    impl ratl_compare {
        pub fn cmp(&self, other: &ratl_compare) -> std::cmp::Ordering {
            self.mCost.partial_cmp(&other.mCost).unwrap_or(std::cmp::Ordering::Equal)
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // this is used to keep track of the constuction state for things that are always constucted
    // ////////////////////////////////////////////////////////////////////////////////////////
    pub struct bits_true;

    impl bits_true {
        pub fn clear(&self) {}
        pub fn set(&self) {}
        pub fn set_bit(&self, _i: usize) {}
        pub fn clear_bit(&self, _i: usize) {}
        pub fn at(&self, _i: usize) -> bool {
            true
        }
        pub fn next_bit(&self, start: usize, onBit: bool) -> usize {
            debug_assert!(onBit);
            start
        }
    }

    pub mod storage {
        use super::mem;
        use super::bits_true;
        use super::bits_base;

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // Value Semantics Storage
        // ////////////////////////////////////////////////////////////////////////////////////////
        pub struct value_semantics<T: Clone + Copy, const SIZE: usize> {
            // Trait-based storage; actual implementation would be in containers
        }

        impl<T: Clone + Copy, const SIZE: usize> value_semantics<T, SIZE> {
            pub const CAPACITY: usize = SIZE;
            pub const NEEDS_CONSTRUCT: i32 = 0;
            pub const TOTAL_SIZE: usize = std::mem::size_of::<T>();
            pub const VALUE_SIZE: usize = std::mem::size_of::<T>();
        }

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // Object Semantics Storage
        // ////////////////////////////////////////////////////////////////////////////////////////
        pub struct object_semantics<T, const SIZE: usize> {
            // Trait-based storage
        }

        impl<T, const SIZE: usize> object_semantics<T, SIZE> {
            pub const CAPACITY: usize = SIZE;
            pub const NEEDS_CONSTRUCT: i32 = 1;
            pub const TOTAL_SIZE: usize = std::mem::size_of::<T>();
            pub const VALUE_SIZE: usize = std::mem::size_of::<T>();
        }

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // Virtual Semantics Storage
        // ////////////////////////////////////////////////////////////////////////////////////////
        pub struct virtual_semantics<T, const SIZE: usize, const MAX_CLASS_SIZE: usize> {
            // Trait-based storage
        }

        impl<T, const SIZE: usize, const MAX_CLASS_SIZE: usize>
            virtual_semantics<T, SIZE, MAX_CLASS_SIZE>
        {
            pub const CAPACITY: usize = SIZE;
            pub const NEEDS_CONSTRUCT: i32 = 1;
            pub const TOTAL_SIZE: usize = std::mem::size_of::<T>();
            pub const VALUE_SIZE: usize = MAX_CLASS_SIZE;
        }

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // Value Semantics with Node
        // ////////////////////////////////////////////////////////////////////////////////////////
        pub struct value_semantics_node<T: Clone + Copy, const SIZE: usize, NODE: Clone + Copy> {
            // Trait-based storage with node data
        }

        impl<T: Clone + Copy, const SIZE: usize, NODE: Clone + Copy>
            value_semantics_node<T, SIZE, NODE>
        {
            pub const CAPACITY: usize = SIZE;
            pub const NEEDS_CONSTRUCT: i32 = 0;
            pub const TOTAL_SIZE: usize = std::mem::size_of::<(NODE, T)>();
            pub const VALUE_SIZE: usize = std::mem::size_of::<T>();
        }

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // Object Semantics with Node
        // ////////////////////////////////////////////////////////////////////////////////////////
        pub struct object_semantics_node<T, const SIZE: usize, NODE: Clone + Copy> {
            // Trait-based storage with node data
        }

        impl<T, const SIZE: usize, NODE: Clone + Copy> object_semantics_node<T, SIZE, NODE> {
            pub const CAPACITY: usize = SIZE;
            pub const NEEDS_CONSTRUCT: i32 = 1;
            pub const TOTAL_SIZE: usize = std::mem::size_of::<(NODE, T)>();
            pub const VALUE_SIZE: usize = std::mem::size_of::<T>();
        }

        // ////////////////////////////////////////////////////////////////////////////////////////
        // // Virtual Semantics with Node
        // ////////////////////////////////////////////////////////////////////////////////////////
        pub struct virtual_semantics_node<T, const SIZE: usize, const MAX_CLASS_SIZE: usize, NODE: Clone + Copy>
        {
            // Trait-based storage with node data
        }

        impl<T, const SIZE: usize, const MAX_CLASS_SIZE: usize, NODE: Clone + Copy>
            virtual_semantics_node<T, SIZE, MAX_CLASS_SIZE, NODE>
        {
            pub const CAPACITY: usize = SIZE;
            pub const NEEDS_CONSTRUCT: i32 = 1;
            pub const TOTAL_SIZE: usize = std::mem::size_of::<(NODE, T)>();
            pub const VALUE_SIZE: usize = MAX_CLASS_SIZE;
        }
    }

    // ////////////////////////////////////////////////////////////////////////////////////////
    // // The Array Base Class, used for most containers
    // ////////////////////////////////////////////////////////////////////////////////////////
    pub struct array_base<T> {
        // Placeholder for generic container; would require actual implementation
        _marker: std::marker::PhantomData<T>,
    }

    impl<T> array_base<T> {
        pub const CAPACITY: usize = 0; // Would be T::CAPACITY
        pub const SIZE: usize = 0;     // Would be T::CAPACITY

        pub fn new() -> Self {
            array_base {
                _marker: std::marker::PhantomData,
            }
        }

        pub fn clear(&mut self) {
            // Placeholder
        }
    }
}
