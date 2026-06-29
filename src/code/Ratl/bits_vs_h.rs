////////////////////////////////////////////////////////////////////////////////////////
// RAVEN STANDARD TEMPLATE LIBRARY
//  (c) 2002 Activision
//
//
// Bit Field
// ---------
// The bits class is a bit field of any length which supports all the
// standard bitwize operations in addition to some operators for adding & removing
// individual bits by their integer indicies and a string conversion method.
//
//
//
// NOTES:
// - The SIZE template variable determines how many BITS are available in this template,
// not how much memory (number of ints) were used to store it.
//
//
////////////////////////////////////////////////////////////////////////////////////////

use core::mem;
use core::ptr;

// External memory functions from ratl::mem namespace
// In C++, these are defined in ratl_common.h
// cpy: memcpy wrapper
// eql: memcmp equality check (returns true if buffers are equal)
extern "C" {
    fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8;
    fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32;
}

// Helper functions that wrap the C memory functions
#[allow(dead_code)]
mod mem_helpers {
    use super::*;

    // Copy memory (wrapper around memcpy)
    #[inline]
    pub fn cpy(dest: *mut u8, src: *const u8, count: usize) {
        unsafe {
            memcpy(dest, src, count);
        }
    }

    // Compare memory for equality (wrapper around memcmp)
    #[inline]
    pub fn eql(buf1: *const u8, buf2: *const u8, count: usize) -> bool {
        unsafe { memcmp(buf1 as *const u8, buf2 as *const u8, count) == 0 }
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Bit Field Class
//
// This is a template in C++: template <int SZ> class bits_vs : public bits_base<SZ>
// Since Rust doesn't support compile-time array size parameters in the same way,
// we provide this as a generic struct parameterized by SIZE.
// In practice, this would be specialized for specific bit counts.
////////////////////////////////////////////////////////////////////////////////////////

#[allow(non_camel_case_types)]
pub struct bits_vs {
    ////////////////////////////////////////////////////////////////////////////////////
    // Constants (from bits_base template)
    ////////////////////////////////////////////////////////////////////////////////////
    // These are calculated from SIZE at specialization time in C++.
    // For now, we use 256 u32 elements which covers up to 8192 bits.
    // The constants are encoded as consts in the implementation.

    ////////////////////////////////////////////////////////////////////////////////////
    // Data - The bit storage array
    ////////////////////////////////////////////////////////////////////////////////////
    // In C++: unsigned int mV[ARRAY_SIZE];
    // where ARRAY_SIZE = ((SIZE + 31) / 32)
    mV: [u32; 256],
}

impl bits_vs {
    ////////////////////////////////////////////////////////////////////////////////////
    // Template Constants - These would be parameterized by SIZE in C++
    ////////////////////////////////////////////////////////////////////////////////////
    const BITS_SHIFT: usize = 5; // 5. Such A Nice Number
    const BITS_INT_SIZE: usize = 32; // Size Of A Single Word
    const BITS_AND: usize = (Self::BITS_INT_SIZE - 1); // Used For And Operation (31)
    // ARRAY_SIZE would be calculated as ((SIZE + BITS_AND) / BITS_INT_SIZE)
    const ARRAY_SIZE: usize = 256; // Number of u32 words used (tuned for typical usage)
    const BYTE_SIZE: usize = Self::ARRAY_SIZE * 4; // Number of bytes used

    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    // SIZE and CAPACITY would be set to the template parameter SZ
    pub const SIZE: usize = 8192; // In C++: template parameter SZ, often set to ARRAY_SIZE*32

    ////////////////////////////////////////////////////////////////////////////////////
    // Call This Function To Set All Bits Beyond SIZE to Zero
    ////////////////////////////////////////////////////////////////////////////////////
    fn clear_trailing_bits(&mut self) {
        for i in Self::SIZE..Self::ARRAY_SIZE * Self::BITS_INT_SIZE {
            self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Standard Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new(init: bool, init_value: bool) -> Self {
        let mut bits = Self {
            mV: [0u32; 256],
        };
        if init {
            if init_value {
                bits.set();
            } else {
                bits.clear();
            }
        }
        bits
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy Constructor (clone-like behavior)
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn from_bits(other: &bits_vs) -> Self {
        let mut bits = Self {
            mV: [0u32; 256],
        };
        mem_helpers::cpy(
            bits.mV.as_mut_ptr() as *mut u8,
            other.mV.as_ptr() as *const u8,
            Self::BYTE_SIZE,
        );
        bits
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // String Constructor (Format: "100010100101")
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn from_str(s: &str) -> Self {
        let mut bits = Self::new(true, false);

        for (b, ch) in s.chars().enumerate() {
            if b >= Self::SIZE {
                break; // Reached The End Of The Valid Bit Range
            }
            if ch == '1' {
                bits.set_bit(b); // Found A True Bit
            }
        }
        bits
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Checks If There Are Any Values At All In This Bit Field (Same as operator !())
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        for i in 0..Self::ARRAY_SIZE {
            if self.mV[i] != 0 {
                return false;
            }
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Get The Number Of Bits Represented Here
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn size(&self) -> usize {
        Self::SIZE
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Execute A Bitwise Flip On All The Bits
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn invert(&mut self) {
        for i in 0..Self::ARRAY_SIZE {
            self.mV[i] = !self.mV[i];
        }
        self.clear_trailing_bits();
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Query (get_bit returns bool for a single bit)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn get_bit(&self, i: usize) -> bool {
        // If you hit this assert, then you are trying
        // to query a bit that goes beyond the number
        // of bits this object can hold.
        //--------------------------------------------
        debug_assert!(i < Self::SIZE);
        ((self.mV[i >> Self::BITS_SHIFT] & (1 << (i & Self::BITS_AND))) != 0)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Checks If There Are Any Values At All In This Bit Field (operator !())
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn is_empty(&self) -> bool {
        self.empty()
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Equality Operator
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn eq(&self, other: &bits_vs) -> bool {
        mem_helpers::eql(
            self.mV.as_ptr() as *const u8,
            other.mV.as_ptr() as *const u8,
            Self::BYTE_SIZE,
        )
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // InEquality Operator
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ne(&self, other: &bits_vs) -> bool {
        !self.eq(other)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Or In From Another Bits Object
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn or_assign(&mut self, other: &bits_vs) {
        for i in 0..Self::ARRAY_SIZE {
            self.mV[i] |= other.mV[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // And In From Another Bits Object
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn and_assign(&mut self, other: &bits_vs) {
        for i in 0..Self::ARRAY_SIZE {
            self.mV[i] &= other.mV[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // xor In From Another Bits Object
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn xor_assign(&mut self, other: &bits_vs) {
        for i in 0..Self::ARRAY_SIZE {
            self.mV[i] ^= other.mV[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator (copy from another bits object)
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn assign(&mut self, other: &bits_vs) {
        mem_helpers::cpy(
            self.mV.as_mut_ptr() as *mut u8,
            other.mV.as_ptr() as *const u8,
            Self::BYTE_SIZE,
        );
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // set_bit - Set a bit to 1
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn set_bit(&mut self, i: usize) {
        debug_assert!(i < Self::SIZE);
        self.mV[i >> Self::BITS_SHIFT] |= 1 << (i & Self::BITS_AND);
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // clear_bit - Clear a bit to 0
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn clear_bit(&mut self, i: usize) {
        debug_assert!(i < Self::SIZE);
        self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // clear - Clear all bits
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn clear(&mut self) {
        for word in &mut self.mV[..Self::ARRAY_SIZE] {
            *word = 0;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // set - Set all bits to 1
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn set(&mut self) {
        for word in &mut self.mV[..Self::ARRAY_SIZE] {
            *word = 0xffffffffu32;
        }
        self.clear_trailing_bits();
    }
}

// Implement standard Rust traits for bits_vs

impl Clone for bits_vs {
    fn clone(&self) -> Self {
        bits_vs::from_bits(self)
    }
}

impl PartialEq for bits_vs {
    fn eq(&self, other: &Self) -> bool {
        bits_vs::eq(self, other)
    }
}

impl Eq for bits_vs {}

impl Default for bits_vs {
    fn default() -> Self {
        bits_vs::new(true, false)
    }
}
