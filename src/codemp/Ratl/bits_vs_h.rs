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

use std::mem;

////////////////////////////////////////////////////////////////////////////////////////
// this is a simplified version of bits_vs
////////////////////////////////////////////////////////////////////////////////////////
pub struct BitsBase<const SZ: usize> {
    // Protected enum values as const generics/associated constants
    // BITS_SHIFT		= 5,									// 5.  Such A Nice Number
    // BITS_INT_SIZE	= 32,									// Size Of A Single Word
    // BITS_AND		= (BITS_INT_SIZE - 1),					// Used For And Operation
    // ARRAY_SIZE		= ((SZ + BITS_AND)/(BITS_INT_SIZE)),	// Num Words Used
    // BYTE_SIZE		= (ARRAY_SIZE*sizeof(unsigned int)),	// Num Bytes Used

    ////////////////////////////////////////////////////////////////////////////////////
    // Data
    ////////////////////////////////////////////////////////////////////////////////////
    mV: [u32; Self::ARRAY_SIZE],
}

impl<const SZ: usize> BitsBase<SZ> {
    const BITS_SHIFT: u32 = 5;
    const BITS_INT_SIZE: usize = 32;
    const BITS_AND: usize = Self::BITS_INT_SIZE - 1;
    const ARRAY_SIZE: usize = (SZ + Self::BITS_AND) / Self::BITS_INT_SIZE;
    const BYTE_SIZE: usize = Self::ARRAY_SIZE * std::mem::size_of::<u32>();

    pub const SIZE: usize = SZ;
    pub const CAPACITY: usize = SZ;

    pub fn new(init: bool, init_value: bool) -> Self {
        let mut bits = BitsBase {
            mV: [0u32; Self::ARRAY_SIZE],
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

    pub fn clear(&mut self) {
        for i in 0..Self::ARRAY_SIZE {
            self.mV[i] = 0;
        }
    }

    pub fn set(&mut self) {
        for i in 0..Self::ARRAY_SIZE {
            self.mV[i] = 0xffffffff;
        }
    }

    pub fn set_bit(&mut self, i: usize) {
        assert!(i >= 0 && i < Self::SIZE);
        self.mV[i >> Self::BITS_SHIFT] |= 1 << (i & Self::BITS_AND);
    }

    pub fn clear_bit(&mut self, i: usize) {
        assert!(i >= 0 && i < Self::SIZE);
        self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
    }

    pub fn mark_bit(&mut self, i: usize, set: bool) {
        assert!(i >= 0 && i < Self::SIZE);
        if set {
            self.mV[i >> Self::BITS_SHIFT] |= 1 << (i & Self::BITS_AND);
        } else {
            self.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
        }
    }

    pub fn index(&self, i: usize) -> bool {
        assert!(i >= 0 && i < Self::SIZE);
        (self.mV[i >> Self::BITS_SHIFT] & (1 << (i & Self::BITS_AND))) != 0
    }

    pub fn next_bit(&self, start: usize, on_bit: bool) -> usize {
        assert!(start >= 0 && start <= Self::SIZE);
        if start >= Self::SIZE {
            return Self::SIZE;
        }
        // Get The Word Which Contains The Start Bit & Mask Out Everything Before The Start Bit
        //--------------------------------------------------------------------------------------
        let mut v = self.mV[start >> Self::BITS_SHIFT];
        if !on_bit {
            v = !v;
        }
        let mut v = v >> (start & 31);
        let mut current_start = start;

        // Search For The First Non Zero Word In The Array
        //-------------------------------------------------
        while v == 0 {
            current_start = (current_start & (!(Self::BITS_INT_SIZE - 1))) + Self::BITS_INT_SIZE;
            if current_start >= Self::SIZE {
                return Self::SIZE;
            }
            v = self.mV[current_start >> Self::BITS_SHIFT];
            if !on_bit {
                v = !v;
            }
        }

        // So, We've Found A Non Zero Word, So Start Masking Against Parts To Skip Over Bits
        //-----------------------------------------------------------------------------------
        if (v & 0xffff) == 0 {
            current_start += 16;
            v >>= 16;
        }
        if (v & 0xff) == 0 {
            current_start += 8;
            v >>= 8;
        }
        if (v & 0xf) == 0 {
            current_start += 4;
            v >>= 4;
        }

        // Time To Search Each Bit
        //-------------------------
        while (v & 1) == 0 {
            current_start += 1;
            v >>= 1;
        }
        if current_start >= Self::SIZE {
            return Self::SIZE;
        }
        current_start
    }
}

////////////////////////////////////////////////////////////////////////////////////////
// The Bit Field Class
////////////////////////////////////////////////////////////////////////////////////////
pub struct BitsVs<const SZ: usize> {
    base: BitsBase<SZ>,
}

impl<const SZ: usize> BitsVs<SZ> {
    const BITS_SHIFT: u32 = 5;
    const BITS_INT_SIZE: usize = 32;
    const BITS_AND: usize = Self::BITS_INT_SIZE - 1;
    const ARRAY_SIZE: usize = (SZ + Self::BITS_AND) / Self::BITS_INT_SIZE;
    const BYTE_SIZE: usize = Self::ARRAY_SIZE * std::mem::size_of::<u32>();

    ////////////////////////////////////////////////////////////////////////////////////
    // Capacity Enum
    ////////////////////////////////////////////////////////////////////////////////////
    pub const SIZE: usize = SZ;
    pub const CAPACITY: usize = SZ;

    ////////////////////////////////////////////////////////////////////////////////////
    // Call This Function To Set All Bits Beyond SIZE to Zero
    ////////////////////////////////////////////////////////////////////////////////////
    fn clear_trailing_bits(&mut self) {
        for i in Self::SIZE..(Self::ARRAY_SIZE * Self::BITS_INT_SIZE) {
            self.base.mV[i >> Self::BITS_SHIFT] &= !(1 << (i & Self::BITS_AND));
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Standard Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn new(init: bool, init_value: bool) -> Self {
        BitsVs {
            base: BitsBase::new(init, init_value),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // Copy Constructor
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn from_copy(b: &BitsVs<SZ>) -> Self {
        let mut new_bits = BitsVs {
            base: BitsBase::new(false, false),
        };
        for i in 0..Self::ARRAY_SIZE {
            new_bits.base.mV[i] = b.base.mV[i];
        }
        new_bits
    }

    ////////////////////////////////////////////////////////////////////////////////////
    // String Constructor (Format: "100010100101")
    ////////////////////////////////////////////////////////////////////////////////////
    pub fn from_str(s: &str) -> Self {
        let mut bits = BitsVs {
            base: BitsBase::new(false, false),
        };
        bits.clear();

        for (b, ch) in s.bytes().enumerate() {
            if b >= Self::SIZE {
                break;
            }
            if ch == b'1' {
                bits.set_bit(b);
            }
        }

        bits
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Checks If There Are Any Values At All In This Bit Field (Same as operator !())
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn empty(&self) -> bool {
        for i in 0..Self::ARRAY_SIZE {
            if self.base.mV[i] != 0 {
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
            self.base.mV[i] = !self.base.mV[i];
        }
        self.clear_trailing_bits();
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Query
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn get_bit(&self, i: usize) -> bool {
        // If you hit this assert, then you are trying
        // to query a bit that goes beyond the number
        // of bits this object can hold.
        //--------------------------------------------
        assert!(i >= 0 && i < Self::SIZE);
        (self.base.mV[i >> Self::BITS_SHIFT] & (1 << (i & Self::BITS_AND))) != 0
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Checks If There Are Any Values At All In This Bit Field
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn is_empty(&self) -> bool {
        self.empty()
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Equality Operator
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn eq(&self, b: &BitsVs<SZ>) -> bool {
        for i in 0..Self::ARRAY_SIZE {
            if self.base.mV[i] != b.base.mV[i] {
                return false;
            }
        }
        true
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // InEquality Operator
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn ne(&self, b: &BitsVs<SZ>) -> bool {
        !self.eq(b)
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Or In From Another Bits Object
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn or_assign(&mut self, b: &BitsVs<SZ>) {
        for i in 0..Self::ARRAY_SIZE {
            self.base.mV[i] |= b.base.mV[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // And In From Another Bits Object
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn and_assign(&mut self, b: &BitsVs<SZ>) {
        for i in 0..Self::ARRAY_SIZE {
            self.base.mV[i] &= b.base.mV[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // xor In From Another Bits Object
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn xor_assign(&mut self, b: &BitsVs<SZ>) {
        for i in 0..Self::ARRAY_SIZE {
            self.base.mV[i] ^= b.base.mV[i];
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////
    // Assignment Operator
    ////////////////////////////////////////////////////////////////////////////////////////
    pub fn assign(&mut self, b: &BitsVs<SZ>) {
        for i in 0..Self::ARRAY_SIZE {
            self.base.mV[i] = b.base.mV[i];
        }
    }

    pub fn clear(&mut self) {
        self.base.clear();
    }

    pub fn set_bit(&mut self, i: usize) {
        self.base.set_bit(i);
    }
}

impl<const SZ: usize> Clone for BitsVs<SZ> {
    fn clone(&self) -> Self {
        BitsVs::from_copy(self)
    }
}

impl<const SZ: usize> PartialEq for BitsVs<SZ> {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

impl<const SZ: usize> Eq for BitsVs<SZ> {}

impl<const SZ: usize> std::ops::BitOrAssign for BitsVs<SZ> {
    fn bitor_assign(&mut self, rhs: Self) {
        self.or_assign(&rhs);
    }
}

impl<const SZ: usize> std::ops::BitAndAssign for BitsVs<SZ> {
    fn bitand_assign(&mut self, rhs: Self) {
        self.and_assign(&rhs);
    }
}

impl<const SZ: usize> std::ops::BitXorAssign for BitsVs<SZ> {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.xor_assign(&rhs);
    }
}
