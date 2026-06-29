/*
 * UNPUBLISHED -- Rights  reserved  under  the  copyright  laws  of the
 * United States.  Use  of a copyright notice is precautionary only and
 * does not imply publication or disclosure.
 *
 * THIS DOCUMENTATION CONTAINS CONFIDENTIAL AND PROPRIETARY INFORMATION
 * OF    VICARIOUS   VISIONS,  INC.    ANY  DUPLICATION,  MODIFICATION,
 * DISTRIBUTION, OR DISCLOSURE IS STRICTLY PROHIBITED WITHOUT THE PRIOR
 * EXPRESS WRITTEN PERMISSION OF VICARIOUS VISIONS, INC.
 */

/*

AUTHOR: Dave Calvin
CREATED: 2002-05-07

SParse ARray Compressor.  Given an array, this class reduces the memory
needed to store the array by eliminating the most-frequently used element.
The remaining elements are increased in size by one integer.

If the compressed data would be larger than the original data, the
original data is stored as is.

Compression is O(2N) where N is the number of elements to compress.

Decompression is O(log M + N) where M is the number of elements after
compression (CompressedLength()) and N is the number of elements to decompress.
Decompression is O(1) when the same or smaller amount of data is requested as
the last decompression.

The pointer returned by Decompress() is valid until the class is destroyed
or a new call is made to Compress() or Decompress().

Elements must define operator==, operator!=, and sizeof.

*/

use core::ffi::c_void;
use std::mem;

// Bigger than a short, smaller than an int.
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct NotSoShort {
    pub bytes: [u8; 3],
}

impl NotSoShort {
    pub fn new() -> Self {
        NotSoShort { bytes: [0; 3] }
    }

    pub fn from_u32(source: u32) -> Self {
        let mut result = NotSoShort { bytes: [0; 3] };
        #[cfg(target_endian = "big")]
        {
            result.bytes[2] = (source & 0xFF) as u8;
            result.bytes[1] = ((source >> 8) & 0xFF) as u8;
            result.bytes[0] = ((source >> 16) & 0xFF) as u8;
        }
        #[cfg(not(target_endian = "big"))]
        {
            result.bytes[0] = (source & 0xFF) as u8;
            result.bytes[1] = ((source >> 8) & 0xFF) as u8;
            result.bytes[2] = ((source >> 16) & 0xFF) as u8;
        }
        result
    }

    pub fn get_value(&self) -> u32 {
        #[cfg(target_endian = "big")]
        {
            ((self.bytes[0] as u32) << 16) | ((self.bytes[1] as u32) << 8) | (self.bytes[2] as u32)
        }
        #[cfg(not(target_endian = "big"))]
        {
            ((self.bytes[2] as u32) << 16) | ((self.bytes[1] as u32) << 8) | (self.bytes[0] as u32)
        }
    }

    pub fn eq_u32(&self, cmp: u32) -> bool {
        #[cfg(target_endian = "big")]
        {
            unsafe {
                let ptr = self.bytes.as_ptr() as *const u32;
                cmp == ((*ptr >> 8) & 0xFFFFFF)
            }
        }
        #[cfg(not(target_endian = "big"))]
        {
            unsafe {
                let ptr = self.bytes.as_ptr() as *const u32;
                cmp == (*ptr & 0x00FFFFFF)
            }
        }
    }

    pub fn lt_u32(&self, cmp: u32) -> bool {
        let mut tmp: u32 = unsafe {
            let ptr = self.bytes.as_ptr() as *const u32;
            *ptr
        };
        #[cfg(target_endian = "big")]
        {
            tmp >>= 8;
        }
        #[cfg(not(target_endian = "big"))]
        {
            tmp &= 0x00FFFFFF;
        }
        tmp < cmp
    }

    pub fn le_u32(&self, cmp: u32) -> bool {
        let mut tmp: u32 = unsafe {
            let ptr = self.bytes.as_ptr() as *const u32;
            *ptr
        };
        #[cfg(target_endian = "big")]
        {
            tmp >>= 8;
        }
        #[cfg(not(target_endian = "big"))]
        {
            tmp &= 0x00FFFFFF;
        }
        tmp <= cmp
    }

    pub fn gt_u32(&self, cmp: u32) -> bool {
        let mut tmp: u32 = unsafe {
            let ptr = self.bytes.as_ptr() as *const u32;
            *ptr
        };
        #[cfg(target_endian = "big")]
        {
            tmp >>= 8;
        }
        #[cfg(not(target_endian = "big"))]
        {
            tmp &= 0x00FFFFFF;
        }
        tmp > cmp
    }
}

// Compressed data is made up of these elements.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SPARCElement<T, U> {
    pub data: T,
    pub offset: U,
}

pub fn SPARC_SWAP32(x: u32, do_swap: bool) -> u32 {
    if do_swap {
        return (((x & 0xff000000) >> 24)
            + ((x & 0x00ff0000) >> 8)
            + ((x & 0x0000ff00) << 8)
            + ((x & 0x000000ff) << 24)) as u32;
    }
    x
}

pub fn SPARC_SWAP24(mut x: NotSoShort, do_swap: bool) -> NotSoShort {
    if do_swap {
        x.bytes[0] ^= x.bytes[2];
        x.bytes[2] ^= x.bytes[0];
        x.bytes[0] ^= x.bytes[2];
    }
    x
}

pub fn SPARC_SWAP16(x: u16, do_swap: bool) -> u16 {
    if do_swap {
        return ((((x & 0xff00) >> 8) + ((x & 0x00ff) << 8)) & 0xFFFF) as u16;
    }
    x
}

// The core of the SPARC system.  T is the data type to be compressed.
// U is the data type needed to store offsets information in the compressed
// data.  Smaller U makes for better compression but bigger data requires
// larger U.
pub struct SPARCCore<T, U> {
    // Using compression or just storing clear data?
    compression_used: bool,

    // Compressed data and its length.
    compressed_data: *mut SPARCElement<T, U>,
    compressed_length: u32,

    // Decompression cache.
    decompressed_data: *mut T,
    decompressed_offset: u32,
    decompressed_length: u32,

    // Element which was removed to compress.
    removed_element: T,

    // Length of original data before compression.
    original_length: u32,

    // Memory allocators.
    allocator: Option<fn(u32) -> *mut c_void>,
    deallocator: Option<fn(*mut c_void)>,
}

impl<T: Copy + PartialEq, U: Copy> SPARCCore<T, U> {
    pub fn new() -> Self {
        SPARCCore {
            compression_used: false,
            compressed_data: std::ptr::null_mut(),
            compressed_length: 0,
            decompressed_data: std::ptr::null_mut(),
            decompressed_offset: 0,
            decompressed_length: 0,
            removed_element: unsafe { mem::zeroed() },
            original_length: 0,
            allocator: None,
            deallocator: None,
        }
    }

    // Destroy all allocated memory.
    fn cleanup(&mut self) {
        if !self.compressed_data.is_null() {
            if let Some(dealloc) = self.deallocator {
                dealloc(self.compressed_data as *mut c_void);
            } else {
                unsafe {
                    let _ = Box::from_raw(self.compressed_data);
                }
            }
            self.compressed_data = std::ptr::null_mut();
        }

        if !self.decompressed_data.is_null() {
            if let Some(dealloc) = self.deallocator {
                dealloc(self.decompressed_data as *mut c_void);
            } else {
                unsafe {
                    let _ = Box::from_raw(self.decompressed_data);
                }
            }
            self.decompressed_data = std::ptr::null_mut();
        }
    }

    fn init(&mut self) {
        self.compression_used = false;
        self.compressed_data = std::ptr::null_mut();
        self.original_length = 0;
        self.compressed_length = 0;
        self.decompressed_data = std::ptr::null_mut();
        self.decompressed_offset = 0;
        self.decompressed_length = 0;
    }

    // Binary search for the compressed element most closely matching 'offset'.
    fn find_decomp_start(&self, offset: u32) -> *mut SPARCElement<T, U> {
        let mut start_point = self.compressed_length / 2;
        let mut divisor = 4;
        loop {
            unsafe {
                if (*self.compressed_data.add(start_point as usize)).offset <= offset.into()
                    && (*self.compressed_data.add((start_point + 1) as usize)).offset
                        > offset.into()
                {
                    if (*self.compressed_data.add(start_point as usize)).offset == offset.into()
                    {
                        return self.compressed_data.add(start_point as usize);
                    } else {
                        return self.compressed_data.add((start_point + 1) as usize);
                    }
                }

                let mut leap = self.compressed_length / divisor;
                if leap < 1 {
                    leap = 1;
                } else {
                    divisor *= 2;
                }
                if (*self.compressed_data.add(start_point as usize)).offset > offset.into() {
                    start_point -= leap;
                } else {
                    start_point += leap;
                }
            }
        }
    }

    pub fn set_allocator(&mut self, alloc: fn(u32) -> *mut c_void, dealloc: fn(*mut c_void)) {
        self.allocator = Some(alloc);
        self.deallocator = Some(dealloc);
    }

    // Just store the array without compression.
    pub fn store(&mut self, array: *const T, length: u32) -> u32 {
        // Destroy old data.
        self.cleanup();
        self.init();

        // Allocate memory and copy array.
        let size_bytes = (length as usize) * mem::size_of::<T>();
        if let Some(alloc) = self.allocator {
            self.decompressed_data = alloc(length) as *mut T;
        } else {
            let layout = std::alloc::Layout::array::<T>(length as usize)
                .expect("allocation layout");
            self.decompressed_data = unsafe { std::alloc::alloc(layout) as *mut T };
        }
        unsafe {
            std::ptr::copy_nonoverlapping(array, self.decompressed_data, length as usize);
        }
        self.compressed_length = length;

        // Set length.
        self.original_length = length;

        self.compressed_size()
    }

    // Load compressed data directly.
    pub fn load(&mut self, mut array: *const c_void, length: u32) -> u32 {
        // Destroy old data.
        self.cleanup();
        self.init();

        unsafe {
            // Restore some attributes.
            self.compression_used = *(array as *const u8) != 0;
            array = (array as *const u8).add(1) as *const c_void;

            // assert(sizeof(T) == 1); // For now only support characters.
            self.removed_element = *(array as *const T);
            array = (array as *const u8).add(mem::size_of::<T>()) as *const c_void;

            self.original_length = *(array as *const u32);
            array = (array as *const u8).add(mem::size_of::<u32>()) as *const c_void;

            self.compressed_length = *(array as *const u32);
            array = (array as *const u8).add(mem::size_of::<u32>()) as *const c_void;

            // Allocate memory and copy array.
            if self.compression_used {
                let elem_size = mem::size_of::<SPARCElement<T, U>>();
                if let Some(alloc) = self.allocator {
                    self.compressed_data = alloc(self.compressed_length) as *mut SPARCElement<T, U>;
                } else {
                    let layout = std::alloc::Layout::array::<SPARCElement<T, U>>(
                        self.compressed_length as usize,
                    )
                    .expect("allocation layout");
                    self.compressed_data = std::alloc::alloc(layout) as *mut SPARCElement<T, U>;
                }
                std::ptr::copy_nonoverlapping(
                    array as *const SPARCElement<T, U>,
                    self.compressed_data,
                    self.compressed_length as usize,
                );
            } else {
                if let Some(alloc) = self.allocator {
                    self.decompressed_data = alloc(self.compressed_length) as *mut T;
                } else {
                    let layout = std::alloc::Layout::array::<T>(self.compressed_length as usize)
                        .expect("allocation layout");
                    self.decompressed_data = std::alloc::alloc(layout) as *mut T;
                }
                std::ptr::copy_nonoverlapping(
                    array as *const T,
                    self.decompressed_data,
                    self.compressed_length as usize,
                );
            }
        }

        self.compressed_size()
    }

    // Save state for later restoration.
    pub fn save(&self, mut array: *mut c_void, length: u32, do_swap: bool) -> u32 {
        // Figure out how much space is needed.
        let mut size = 1 + mem::size_of::<T>() + 2 * mem::size_of::<u32>();

        if self.compression_used {
            size += (self.compressed_length as usize) * mem::size_of::<SPARCElement<T, U>>();
        } else {
            size += (self.compressed_length as usize) * mem::size_of::<T>();
        }

        assert!(length as usize >= size);

        unsafe {
            // Save some attributes.
            *(array as *mut u8) = if self.compression_used { 1 } else { 0 };
            array = (array as *mut u8).add(1) as *mut c_void;

            // assert(sizeof(T) == 1); // For now only support characters.
            *(array as *mut T) = self.removed_element;
            array = (array as *mut u8).add(mem::size_of::<T>()) as *mut c_void;

            *(array as *mut u32) = SPARC_SWAP32(self.original_length, do_swap);
            array = (array as *mut u8).add(mem::size_of::<u32>()) as *mut c_void;

            *(array as *mut u32) = SPARC_SWAP32(self.compressed_length, do_swap);
            array = (array as *mut u8).add(mem::size_of::<u32>()) as *mut c_void;

            // Store compressed data (or uncompressed data if none exists)
            if self.compression_used {
                for i in 0..self.compressed_length {
                    // Copy the data element.  For now only support characters.
                    (*(array as *mut SPARCElement<T, U>).add(i as usize)).data =
                        (*self.compressed_data.add(i as usize)).data;

                    // Copy the offset to the next unique element.
                    if mem::size_of::<U>() == 1 {
                        (*(array as *mut SPARCElement<T, U>).add(i as usize)).offset =
                            (*self.compressed_data.add(i as usize)).offset;
                    } else if mem::size_of::<U>() == 2 {
                        (*(array as *mut SPARCElement<T, u16>).add(i as usize)).offset =
                            SPARC_SWAP16(
                                *((&(*self.compressed_data.add(i as usize)).offset)
                                    as *const U as *const u16),
                                do_swap,
                            );
                    } else if mem::size_of::<U>() == 3 {
                        (*(array as *mut SPARCElement<T, NotSoShort>).add(i as usize)).offset =
                            SPARC_SWAP24(
                                *((&(*self.compressed_data.add(i as usize)).offset)
                                    as *const U as *const NotSoShort),
                                do_swap,
                            );
                    } else if mem::size_of::<U>() == 4 {
                        (*(array as *mut SPARCElement<T, u32>).add(i as usize)).offset =
                            SPARC_SWAP32(
                                *((&(*self.compressed_data.add(i as usize)).offset)
                                    as *const U as *const u32),
                                do_swap,
                            );
                    }
                }
            } else {
                std::ptr::copy_nonoverlapping(
                    self.decompressed_data,
                    array as *mut T,
                    self.compressed_length as usize,
                );
            }
        }

        size as u32
    }

    // Compresses this array, returns the compressed size.  Compresses
    // by eliminating the given element.
    pub fn compress(&mut self, array: *const T, length: u32, removal: T) -> u32 {
        let mut num_remove = 0;

        // Destroy old data.
        self.cleanup();
        self.init();

        // Count number of elements to remove.  Can't remove first or
        // last element (prevents boundary conditions).
        for i in 1..(length - 1) {
            unsafe {
                if *array.add(i as usize) == removal {
                    num_remove += 1;
                }
            }
        }

        self.compressed_length = length - num_remove;
        self.original_length = length;

        // If we're going to allocate more memory than was originally used,
        // just store the data.
        if (mem::size_of::<SPARCElement<T, U>>() * (self.compressed_length as usize))
            >= (mem::size_of::<T>() * (length as usize))
        {
            return self.store(array, length);
        }

        // Allocate memory for compressed elements.
        if let Some(alloc) = self.allocator {
            self.compressed_data = alloc(self.compressed_length) as *mut SPARCElement<T, U>;
        } else {
            let layout = std::alloc::Layout::array::<SPARCElement<T, U>>(
                self.compressed_length as usize,
            )
            .expect("allocation layout");
            self.compressed_data = unsafe { std::alloc::alloc(layout) as *mut SPARCElement<T, U> };
        }
        self.compression_used = true;

        // Fill compressed array.  First and last elements go in no matter
        // what.
        unsafe {
            (*self.compressed_data).data = *array;
            (*self.compressed_data).offset = std::mem::zeroed(); // 0 offset
            let mut compress = self.compressed_data.add(1);
            for i in 1..(length - 1) {
                if *array.add(i as usize) != removal {
                    (*compress).data = *array.add(i as usize);
                    (*compress).offset = std::mem::zeroed(); // Will be set to i
                    compress = compress.add(1);
                }
            }
            (*compress).data = *array.add((length - 1) as usize);
            (*compress).offset = std::mem::zeroed(); // Will be set to length-1
        }

        // Store removal value for decompression purposes.
        self.removed_element = removal;

        // Store original length for bounds checking.
        self.original_length = length;

        // Return the compressed size.
        self.compressed_size()
    }

    // Get the compressed data size in bytes, or 0 if nothing stored.
    pub fn compressed_size(&self) -> u32 {
        ((self.compressed_length as usize) * mem::size_of::<SPARCElement<T, U>>()) as u32
    }

    // Get the decompressed data starting at offset and ending at
    // offset + length.  Returns NULL on error.
    pub fn decompress(&mut self, offset: u32, length: u32) -> *const T {
        // If data isn't compressed, just return a pointer.
        if !self.compression_used {
            unsafe {
                return self.decompressed_data.add(offset as usize);
            }
        }

        // If last decompression falls within offset and length, just return
        // a pointer.
        if !self.decompressed_data.is_null()
            && self.decompressed_offset <= offset
            && self.decompressed_offset + self.decompressed_length >= offset + length
        {
            unsafe {
                return self.decompressed_data.add((offset - self.decompressed_offset) as usize);
            }
        }

        // Allocate new space for decompression if length has changed.
        if length != self.decompressed_length {
            // Destroy old data first.
            if !self.decompressed_data.is_null() {
                if let Some(dealloc) = self.deallocator {
                    dealloc(self.decompressed_data as *mut c_void);
                } else {
                    unsafe {
                        let _ = Box::from_raw(self.decompressed_data);
                    }
                }
            }

            if let Some(alloc) = self.allocator {
                self.decompressed_data = alloc(length) as *mut T;
            } else {
                let layout =
                    std::alloc::Layout::array::<T>(length as usize).expect("allocation layout");
                self.decompressed_data = unsafe { std::alloc::alloc(layout) as *mut T };
            }
        }
        self.decompressed_offset = offset;
        self.decompressed_length = length;

        // Find position to start decompressing from.
        let mut decomp = self.find_decomp_start(offset);

        if decomp.is_null() {
            // should never happen
            return std::ptr::null();
        }

        // Decompress the data.
        unsafe {
            for i in 0..length {
                if (*decomp).offset == (i + offset).into() {
                    *self.decompressed_data.add(i as usize) = (*decomp).data;
                    decomp = decomp.add(1);
                } else {
                    *self.decompressed_data.add(i as usize) = self.removed_element;
                }
            }
        }

        self.decompressed_data
    }
}

impl<T, U> Drop for SPARCCore<T, U> {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// The user-interface to SPARC.  Automatically selects the best core based
// on data size.
pub struct SPARC<T> {
    core: *mut c_void,
    offset_bytes: u8,

    // Memory allocators.
    allocator: Option<fn(u32) -> *mut c_void>,
    deallocator: Option<fn(*mut c_void)>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Copy + PartialEq + std::fmt::Debug> SPARC<T> {
    pub fn new() -> Self {
        SPARC {
            core: std::ptr::null_mut(),
            offset_bytes: 0,
            allocator: None,
            deallocator: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set_allocator(&mut self, alloc: fn(u32) -> *mut c_void, dealloc: fn(*mut c_void)) {
        self.allocator = Some(alloc);
        self.deallocator = Some(dealloc);
    }

    // Select a core, cast it to the right type and return the size.
    pub fn compressed_size(&self) -> u32 {
        if self.core.is_null() {
            return 0;
        }

        match self.offset_bytes {
            1 => unsafe { (*(self.core as *mut SPARCCore<T, u8>)).compressed_size() },
            2 => unsafe { (*(self.core as *mut SPARCCore<T, u16>)).compressed_size() },
            3 => unsafe { (*(self.core as *mut SPARCCore<T, NotSoShort>)).compressed_size() },
            4 => unsafe { (*(self.core as *mut SPARCCore<T, u32>)).compressed_size() },
            _ => 0,
        }
    }

    // Always use the same core type since we won't be compressing.
    pub fn store(&mut self, array: *const T, length: u32) -> u32 {
        self.release();
        self.offset_bytes = 1;
        let core = Box::new(SPARCCore::<T, u8>::new());
        if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
            unsafe {
                (*(Box::into_raw(core))).set_allocator(alloc, dealloc);
                let ptr = Box::into_raw(Box::new(*(self.core as *mut SPARCCore<T, u8>)));
                self.core = ptr as *mut c_void;
                (*(self.core as *mut SPARCCore<T, u8>)).store(array, length)
            }
        } else {
            unsafe {
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                (*(self.core as *mut SPARCCore<T, u8>)).store(array, length)
            }
        }
    }

    // Load compressed data directly.
    pub fn load(&mut self, array: *const c_void, length: u32) -> u32 {
        self.release();

        unsafe {
            self.offset_bytes = *(array as *const u8);
        }

        match self.offset_bytes {
            1 => unsafe {
                let core = Box::new(SPARCCore::<T, u8>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).load(
                    (array as *const u8).add(1) as *const c_void,
                    length - 1,
                )
            },
            2 => unsafe {
                let core = Box::new(SPARCCore::<T, u16>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).load(
                    (array as *const u8).add(1) as *const c_void,
                    length - 1,
                )
            },
            3 => unsafe {
                let core = Box::new(SPARCCore::<T, NotSoShort>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).load(
                    (array as *const u8).add(1) as *const c_void,
                    length - 1,
                )
            },
            4 => unsafe {
                let core = Box::new(SPARCCore::<T, u32>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).load(
                    (array as *const u8).add(1) as *const c_void,
                    length - 1,
                )
            },
            _ => {
                assert!(false);
                0
            }
        }
    }

    // Save compressed data into array.
    pub fn save(&self, mut array: *mut c_void, length: u32, do_swap: bool) -> u32 {
        unsafe {
            *(array as *mut u8) = self.offset_bytes;
            array = (array as *mut u8).add(1) as *mut c_void;
        }

        match self.offset_bytes {
            1 => unsafe { (*(self.core as *mut SPARCCore<T, u8>)).save(array, length - 1, do_swap) },
            2 => unsafe { (*(self.core as *mut SPARCCore<T, u16>)).save(array, length - 1, do_swap) },
            3 => unsafe {
                (*(self.core as *mut SPARCCore<T, NotSoShort>)).save(array, length - 1, do_swap)
            },
            4 => unsafe { (*(self.core as *mut SPARCCore<T, u32>)).save(array, length - 1, do_swap) },
            _ => {
                assert!(false);
                0
            }
        }
    }

    // Create the smallest core possible for the given data.
    pub fn compress(&mut self, array: *const T, length: u32, removal: T) -> u32 {
        self.release();

        if length < 256 {
            self.offset_bytes = 1;
            unsafe {
                let core = Box::new(SPARCCore::<T, u8>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).compress(array, length, removal)
            }
        } else if length < 65536 {
            self.offset_bytes = 2;
            unsafe {
                let core = Box::new(SPARCCore::<T, u16>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).compress(array, length, removal)
            }
        } else if length < 16777216 {
            self.offset_bytes = 3;
            unsafe {
                let core = Box::new(SPARCCore::<T, NotSoShort>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).compress(array, length, removal)
            }
        } else {
            self.offset_bytes = 4;
            unsafe {
                let core = Box::new(SPARCCore::<T, u32>::new());
                let ptr = Box::into_raw(core);
                self.core = ptr as *mut c_void;
                if let (Some(alloc), Some(dealloc)) = (self.allocator, self.deallocator) {
                    (*(ptr)).set_allocator(alloc, dealloc);
                }
                (*(ptr)).compress(array, length, removal)
            }
        }
    }

    // Cast to the correct core type and decompress.
    pub fn decompress(&mut self, offset: u32, length: u32) -> *const T {
        if self.core.is_null() {
            return std::ptr::null();
        }

        match self.offset_bytes {
            1 => unsafe { (*(self.core as *mut SPARCCore<T, u8>)).decompress(offset, length) },
            2 => unsafe { (*(self.core as *mut SPARCCore<T, u16>)).decompress(offset, length) },
            3 => unsafe {
                (*(self.core as *mut SPARCCore<T, NotSoShort>)).decompress(offset, length)
            },
            4 => unsafe { (*(self.core as *mut SPARCCore<T, u32>)).decompress(offset, length) },
            _ => std::ptr::null(),
        }
    }

    // Destroy all compressed data and the current decompressed buffer.
    pub fn release(&mut self) {
        if !self.core.is_null() {
            match self.offset_bytes {
                1 => unsafe {
                    let _ = Box::from_raw(self.core as *mut SPARCCore<T, u8>);
                },
                2 => unsafe {
                    let _ = Box::from_raw(self.core as *mut SPARCCore<T, u16>);
                },
                3 => unsafe {
                    let _ = Box::from_raw(self.core as *mut SPARCCore<T, NotSoShort>);
                },
                4 => unsafe {
                    let _ = Box::from_raw(self.core as *mut SPARCCore<T, u32>);
                },
                _ => {}
            }
            self.core = std::ptr::null_mut();
        }
    }
}

impl<T> Drop for SPARC<T> {
    fn drop(&mut self) {
        self.release();
    }
}
