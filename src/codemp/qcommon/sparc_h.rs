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

#![allow(non_snake_case)]

use core::ffi::{c_void, c_uint, c_uchar, c_ushort};
use core::mem::{size_of, zeroed};
use core::ptr::{null_mut, addr_of_mut};

#[cfg(target_os = "gamecube")]
const SPARC_BIG_ENDIAN: bool = true;

#[cfg(not(target_os = "gamecube"))]
const SPARC_BIG_ENDIAN: bool = false;

//Bigger than a short, smaller than an int.
#[repr(C, packed(1))]
#[derive(Clone, Copy, Debug)]
pub struct NotSoShort {
    pub bytes: [c_uchar; 3],
}

impl NotSoShort {
    pub fn new() -> Self {
        unsafe { zeroed() }
    }

    pub fn from_uint(source: c_uint) -> Self {
        let mut result: NotSoShort = unsafe { zeroed() };
        #[cfg(target_os = "gamecube")]
        {
            result.bytes[2] = (source & 0xFF) as c_uchar;
            result.bytes[1] = ((source >> 8) & 0xFF) as c_uchar;
            result.bytes[0] = ((source >> 16) & 0xFF) as c_uchar;
        }
        #[cfg(not(target_os = "gamecube"))]
        {
            result.bytes[0] = (source & 0xFF) as c_uchar;
            result.bytes[1] = ((source >> 8) & 0xFF) as c_uchar;
            result.bytes[2] = ((source >> 16) & 0xFF) as c_uchar;
        }
        result
    }

    #[inline]
    pub fn GetValue(&self) -> c_uint {
        #[cfg(target_os = "gamecube")]
        {
            ((self.bytes[0] as c_uint) << 16) | ((self.bytes[1] as c_uint) << 8) | (self.bytes[2] as c_uint)
        }
        #[cfg(not(target_os = "gamecube"))]
        {
            ((self.bytes[2] as c_uint) << 16) | ((self.bytes[1] as c_uint) << 8) | (self.bytes[0] as c_uint)
        }
    }

    #[inline]
    pub fn operator_eq(&self, cmp: c_uint) -> bool {
        #[cfg(target_os = "gamecube")]
        {
            unsafe {
                let ptr = self.bytes.as_ptr() as *const c_uint;
                cmp == ((*ptr) >> 8)
            }
        }
        #[cfg(not(target_os = "gamecube"))]
        {
            unsafe {
                let ptr = self.bytes.as_ptr() as *const c_uint;
                cmp == ((*ptr) & 0x00FFFFFF)
            }
        }
    }

    pub fn operator_lt(&self, cmp: c_uint) -> bool {
        let tmp: c_uint = unsafe {
            let ptr = self.bytes.as_ptr() as *const c_uint;
            let val = *ptr;
            #[cfg(target_os = "gamecube")]
            {
                val >> 8
            }
            #[cfg(not(target_os = "gamecube"))]
            {
                val & 0x00FFFFFF
            }
        };
        tmp < cmp
    }

    pub fn operator_le(&self, cmp: c_uint) -> bool {
        let tmp: c_uint = unsafe {
            let ptr = self.bytes.as_ptr() as *const c_uint;
            let val = *ptr;
            #[cfg(target_os = "gamecube")]
            {
                val >> 8
            }
            #[cfg(not(target_os = "gamecube"))]
            {
                val & 0x00FFFFFF
            }
        };
        tmp <= cmp
    }

    pub fn operator_gt(&self, cmp: c_uint) -> bool {
        let tmp: c_uint = unsafe {
            let ptr = self.bytes.as_ptr() as *const c_uint;
            let val = *ptr;
            #[cfg(target_os = "gamecube")]
            {
                val >> 8
            }
            #[cfg(not(target_os = "gamecube"))]
            {
                val & 0x00FFFFFF
            }
        };
        tmp > cmp
    }
}

//Compressed data is made up of these elements.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SPARCElement<T, U> {
    pub data: T,
    pub offset: U,
}

#[inline]
pub fn SPARC_SWAP32(x: c_uint, doSwap: bool) -> c_uint {
    if doSwap {
        return (((x & 0xff000000) >> 24)
            + ((x & 0x00ff0000) >> 8)
            + ((x & 0x0000ff00) << 8)
            + ((x & 0x000000ff) << 24));
    }
    x
}

#[inline]
pub fn SPARC_SWAP24(mut x: NotSoShort, doSwap: bool) -> NotSoShort {
    if doSwap {
        x.bytes[0] ^= x.bytes[2];
        x.bytes[2] ^= x.bytes[0];
        x.bytes[0] ^= x.bytes[2];
    }
    x
}

#[inline]
pub fn SPARC_SWAP16(x: c_ushort, doSwap: bool) -> c_ushort {
    if doSwap {
        return ((((x & 0xff00) >> 8)
            + ((x & 0x00ff) << 8)) as c_ushort);
    }
    x
}


//The core of the SPARC system.  T is the data type to be compressed.
//U is the data type needed to store offsets information in the compressed
//data.  Smaller U makes for better compression but bigger data requires
//larger U.
pub struct SPARCCore<T, U>
where
    T: Clone + Copy + PartialEq + Default,
    U: Clone + Copy,
{
    //Using compression or just storing clear data?
    compressionUsed: bool,

    //Compressed data and its length.
    compressedData: *mut SPARCElement<T, U>,
    compressedLength: c_uint,

    //Decompression cache.
    decompressedData: *mut T,
    decompressedOffset: c_uint,
    decompressedLength: c_uint,

    //Element which was removed to compress.
    removedElement: T,

    //Length of original data before compression.
    originalLength: c_uint,

    //Memory allocators.
    Allocator: Option<unsafe extern "C" fn(c_uint) -> *mut c_void>,
    Deallocator: Option<unsafe extern "C" fn(*mut c_void)>,
}

impl<T, U> SPARCCore<T, U>
where
    T: Clone + Copy + PartialEq + Default,
    U: Clone + Copy,
{
    pub fn new() -> Self {
        SPARCCore {
            compressionUsed: false,
            compressedData: null_mut(),
            compressedLength: 0,
            decompressedData: null_mut(),
            decompressedOffset: 0,
            decompressedLength: 0,
            removedElement: T::default(),
            originalLength: 0,
            Allocator: None,
            Deallocator: None,
        }
    }

    //Destroy all allocated memory.
    fn Cleanup(&mut self) {
        if !self.compressedData.is_null() {
            if let Some(dealloc) = self.Deallocator {
                unsafe { dealloc(self.compressedData as *mut c_void); }
            } else {
                unsafe { drop(Box::from_raw(self.compressedData)); }
            }
            self.compressedData = null_mut();
        }

        if !self.decompressedData.is_null() {
            if let Some(dealloc) = self.Deallocator {
                unsafe { dealloc(self.decompressedData as *mut c_void); }
            } else {
                unsafe { drop(Box::from_raw(self.decompressedData)); }
            }
            self.decompressedData = null_mut();
        }
    }

    fn Init(&mut self) {
        self.compressionUsed = false;
        self.compressedData = null_mut();
        self.originalLength = 0;
        self.compressedLength = 0;
        self.decompressedData = null_mut();
        self.decompressedOffset = 0;
        self.decompressedLength = 0;
    }


    //Binary search for the compressed element most closely matching 'offset'.
    fn FindDecompStart(&self, offset: c_uint) -> *mut SPARCElement<T, U>
    {
        let mut startPoint: c_uint = self.compressedLength / 2;
        let mut divisor: c_uint = 4;
        let mut leap: c_uint;
        loop {
            unsafe {
                if self.compressedData.add(startPoint as usize).read().offset <= offset &&
                        self.compressedData.add((startPoint + 1) as usize).read().offset > offset {
                    if self.compressedData.add(startPoint as usize).read().offset == offset {
                        return self.compressedData.add(startPoint as usize);
                    } else {
                        return self.compressedData.add((startPoint + 1) as usize);
                    }
                }

                leap = self.compressedLength / divisor;
                if leap < 1 {
                    leap = 1;
                } else {
                    divisor *= 2;
                }
                if self.compressedData.add(startPoint as usize).read().offset > offset {
                    startPoint -= leap;
                } else {
                    startPoint += leap;
                }
            }
        }
    }

    pub fn SetAllocator(&mut self, alloc: Option<unsafe extern "C" fn(c_uint) -> *mut c_void>,
            dealloc: Option<unsafe extern "C" fn(*mut c_void)>) {
        self.Allocator = alloc;
        self.Deallocator = dealloc;
    }

    //Just store the array without compression.
    pub fn Store(&mut self, array: *const T, length: c_uint) -> c_uint {
        //Destroy old data.
        self.Cleanup();
        self.Init();

        //Allocate memory and copy array.
        if let Some(allocator) = self.Allocator {
            unsafe {
                self.decompressedData = allocator((length as usize * size_of::<T>()) as c_uint) as *mut T;
            }
        } else {
            let vec: Box<[T]> = unsafe {
                let mut v = Vec::with_capacity(length as usize);
                core::ptr::copy_nonoverlapping(array, v.as_mut_ptr(), length as usize);
                v.set_len(length as usize);
                v.into_boxed_slice()
            };
            self.decompressedData = Box::into_raw(vec) as *mut T;
        }
        self.compressedLength = length;
        unsafe {
            core::ptr::copy_nonoverlapping(array, self.decompressedData, length as usize);
        }

        //Set length.
        self.originalLength = length;

        self.CompressedSize()
    }

    //Load compressed data directly.
    pub fn Load(&mut self, mut array: *const c_uchar, length: c_uint) -> c_uint {
        //Destroy old data.
        self.Cleanup();
        self.Init();

        //Restore some attributes.
        unsafe {
            self.compressionUsed = (*array) != 0;
            array = array.add(1);
        }

        debug_assert_eq!(size_of::<T>(), 1); //For now only support characters.
        unsafe {
            self.removedElement = *(array as *const T);
            array = array.add(size_of::<T>());
        }

        unsafe {
            self.originalLength = *(array as *const c_uint);
            array = array.add(size_of::<c_uint>());

            self.compressedLength = *(array as *const c_uint);
            array = array.add(size_of::<c_uint>());
        }

        //Allocate memory and copy array.
        if self.compressionUsed {
            if let Some(allocator) = self.Allocator {
                unsafe {
                    self.compressedData = allocator((self.compressedLength as usize * size_of::<SPARCElement<T, U>>()) as c_uint) as *mut SPARCElement<T, U>;
                }
            } else {
                unsafe {
                    let v: Vec<SPARCElement<T, U>> = Vec::with_capacity(self.compressedLength as usize);
                    self.compressedData = Box::into_raw(v.into_boxed_slice()) as *mut SPARCElement<T, U>;
                }
            }
            unsafe {
                core::ptr::copy_nonoverlapping(array as *const SPARCElement<T, U>, self.compressedData,
                    self.compressedLength as usize);
            }
        }
        else {
            if let Some(allocator) = self.Allocator {
                unsafe {
                    self.decompressedData = allocator(
                        (self.compressedLength as usize * size_of::<T>()) as c_uint) as *mut T;
                }
            } else {
                unsafe {
                    let v: Vec<T> = Vec::with_capacity(self.compressedLength as usize);
                    self.decompressedData = Box::into_raw(v.into_boxed_slice()) as *mut T;
                }
            }
            unsafe {
                core::ptr::copy_nonoverlapping(array as *const T, self.decompressedData, self.compressedLength as usize);
            }
        }

        self.CompressedSize()
    }

    //Save state for later restoration.
    pub fn Save(&self, mut array: *mut c_uchar, length: c_uint, doSwap: bool) -> c_uint {
        //Figure out how much space is needed.
        let mut size: c_uint = (size_of::<c_uchar>() + size_of::<T>() +
            size_of::<c_uint>() + size_of::<c_uint>()) as c_uint;

        if self.compressionUsed {
            size += (self.compressedLength as usize * size_of::<SPARCElement<T, U>>()) as c_uint;
        }
        else {
            size += (self.compressedLength as usize * size_of::<T>()) as c_uint;
        }

        debug_assert!(length >= size);

        //Save some attributes.
        unsafe {
            *array = if self.compressionUsed { 1 } else { 0 };
            array = array.add(1);
        }

        debug_assert_eq!(size_of::<T>(), 1); //For now only support characters.
        unsafe {
            *(array as *mut T) = self.removedElement;
            array = array.add(size_of::<T>());
        }

        unsafe {
            *(array as *mut c_uint) = SPARC_SWAP32(self.originalLength, doSwap);
            array = array.add(size_of::<c_uint>());

            *(array as *mut c_uint) = SPARC_SWAP32(self.compressedLength, doSwap);
            array = array.add(size_of::<c_uint>());
        }

        //Store compressed data (or uncompressed data if none exists)
        if self.compressionUsed {
            for i in 0..self.compressedLength {
                unsafe {
                    //Copy the data element.  For now only support characters.
                    (array as *mut SPARCElement<T, U>).add(i as usize).write(
                        SPARCElement {
                            data: self.compressedData.add(i as usize).read().data,
                            offset: self.compressedData.add(i as usize).read().offset,
                        }
                    );

                    //Copy the offset to the next unique element.
                    if size_of::<U>() == 1 {
                        (array as *mut SPARCElement<T, U>).add(i as usize).write(
                            SPARCElement {
                                data: self.compressedData.add(i as usize).read().data,
                                offset: self.compressedData.add(i as usize).read().offset,
                            }
                        );
                    }
                    else if size_of::<U>() == 2 {
                        (array as *mut SPARCElement<T, c_ushort>).add(i as usize).write(
                            SPARCElement {
                                data: self.compressedData.add(i as usize).read().data,
                                offset: SPARC_SWAP16(
                                    *((&self.compressedData.add(i as usize).read().offset) as *const U as *const c_ushort),
                                    doSwap),
                            }
                        );
                    }
                    else if size_of::<U>() == 3 {
                        (array as *mut SPARCElement<T, NotSoShort>).add(i as usize).write(
                            SPARCElement {
                                data: self.compressedData.add(i as usize).read().data,
                                offset: SPARC_SWAP24(
                                    *((&self.compressedData.add(i as usize).read().offset) as *const U as *const NotSoShort),
                                    doSwap),
                            }
                        );
                    }
                    else if size_of::<U>() == 4 {
                        (array as *mut SPARCElement<T, c_uint>).add(i as usize).write(
                            SPARCElement {
                                data: self.compressedData.add(i as usize).read().data,
                                offset: SPARC_SWAP32(
                                    *((&self.compressedData.add(i as usize).read().offset) as *const U as *const c_uint),
                                    doSwap),
                            }
                        );
                    }
                }
            }
        }
        else {
            unsafe {
                core::ptr::copy_nonoverlapping(self.decompressedData, array as *mut T, self.compressedLength as usize);
            }
        }

        size
    }

    //Compresses this array, returns the compressed size.  Compresses
    //by eliminating the given element.
    pub fn Compress(&mut self, array: *const T, length: c_uint, removal: T) -> c_uint {

        let mut i: c_uint;
        let mut numRemove: c_uint = 0;
        let mut compress: *mut SPARCElement<T, U>;

        //Destroy old data.
        self.Cleanup();
        self.Init();

        //Count number of elements to remove.  Can't remove first or
        //last element (prevents boundary conditions).
        i = 1;
        while i < length - 1 {
            if unsafe { *(array.add(i as usize)) == removal } {
                numRemove += 1;
            }
            i += 1;
        }

        self.compressedLength = length - numRemove;
        self.originalLength = length;

        //If we're going to allocate more memory than was originally used,
        //just store the data.
        if (size_of::<SPARCElement<T, U>>() as c_uint * self.compressedLength) >=
                (size_of::<T>() as c_uint * length) {
            return self.Store(array, length);
        }

        //Allocate memory for compressed elements.
        if let Some(allocator) = self.Allocator {
            unsafe {
                self.compressedData = allocator((self.compressedLength as usize * size_of::<SPARCElement<T, U>>()) as c_uint) as *mut SPARCElement<T, U>;
            }
        } else {
            let v: Vec<SPARCElement<T, U>> = Vec::with_capacity(self.compressedLength as usize);
            self.compressedData = Box::into_raw(v.into_boxed_slice()) as *mut SPARCElement<T, U>;
        }
        self.compressionUsed = true;

        //Fill compressed array.  First and last elements go in no matter
        //what.
        unsafe {
            self.compressedData.write(SPARCElement {
                data: *array,
                offset: 0,
            });
            compress = self.compressedData.add(1);
        }
        i = 1;
        while i < length - 1 {
            unsafe {
                if *array.add(i as usize) != removal {
                    compress.write(SPARCElement {
                        data: *array.add(i as usize),
                        offset: i,
                    });
                    compress = compress.add(1);
                }
            }
            i += 1;
        }
        unsafe {
            compress.write(SPARCElement {
                data: *array.add(i as usize),
                offset: i,
            });
        }

        //Store removal value for decompression purposes.
        self.removedElement = removal;

        //Store original length for bounds checking.
        self.originalLength = length;

        //Return the compressed size.
        self.CompressedSize()
    }


    //Get the compressed data size in bytes, or 0 if nothing stored.
    pub fn CompressedSize(&self) -> c_uint {
        (self.compressedLength as usize * size_of::<SPARCElement<T, U>>()) as c_uint
    }

    //Get the decompressed data starting at offset and ending at
    //offset + length.  Returns NULL on error.
    pub fn Decompress(&mut self, offset: c_uint, length: c_uint) -> *const T {

        let mut decomp: *mut SPARCElement<T, U> = null_mut();
        let mut i: c_uint;

        //If data isn't compressed, just return a pointers.
        if !self.compressionUsed {
            return unsafe { self.decompressedData.add(offset as usize) };
        }

        //If last decompression falls within offset and length, just return
        //a pointer.
        if !self.decompressedData.is_null() && self.decompressedOffset <= offset &&
                self.decompressedOffset + self.decompressedLength >= offset + length {
            return unsafe { self.decompressedData.add((offset - self.decompressedOffset) as usize) };
        }



        //Allocate new space for decompression if length has changed.
        if length != self.decompressedLength {
            //Destroy old data first.
            if !self.decompressedData.is_null() {
                if let Some(dealloc) = self.Deallocator {
                    unsafe { dealloc(self.decompressedData as *mut c_void); }
                } else {
                    unsafe { drop(Box::from_raw(self.decompressedData)); }
                }
            }

            if let Some(allocator) = self.Allocator {
                unsafe {
                    self.decompressedData = allocator((length as usize * size_of::<T>()) as c_uint) as *mut T;
                }
            } else {
                let v: Vec<T> = Vec::with_capacity(length as usize);
                self.decompressedData = Box::into_raw(v.into_boxed_slice()) as *mut T;
            }
        }
        self.decompressedOffset = offset;
        self.decompressedLength = length;

        //Find position to start decompressing from.
        decomp = self.FindDecompStart(offset);

        if decomp.is_null() { //should never happen
            debug_assert!(false);
            return null_mut();
        }

        //Decompress the data.
        i = 0;
        while i < length {
            unsafe {
                if (*decomp).offset == i + offset {
                    self.decompressedData.add(i as usize).write((*decomp).data);
                    decomp = decomp.add(1);
                } else {
                    self.decompressedData.add(i as usize).write(self.removedElement);
                }
            }
            i += 1;
        }

        self.decompressedData
    }
}

impl<T, U> Drop for SPARCCore<T, U>
where
    T: Clone + Copy + PartialEq + Default,
    U: Clone + Copy,
{
    fn drop(&mut self) {
        self.Cleanup();
    }
}


//The user-interface to SPARC.  Automatically selects the best core based
//on data size.
pub struct SPARC<T>
where
    T: Clone + Copy + PartialEq + Default,
{
    core: *mut c_void,
    offsetBytes: c_uchar,

    //Memory allocators.
    Allocator: Option<unsafe extern "C" fn(c_uint) -> *mut c_void>,
    Deallocator: Option<unsafe extern "C" fn(*mut c_void)>,
}

impl<T> SPARC<T>
where
    T: Clone + Copy + PartialEq + Default,
{
    pub fn new() -> Self {
        SPARC {
            core: null_mut(),
            offsetBytes: 0,
            Allocator: None,
            Deallocator: None,
        }
    }

    pub fn SetAllocator(&mut self, alloc: Option<unsafe extern "C" fn(c_uint) -> *mut c_void>,
            dealloc: Option<unsafe extern "C" fn(*mut c_void)>) {
        self.Allocator = alloc;
        self.Deallocator = dealloc;
    }

    //Select a core, cast it to the right type and return the size.
    pub fn CompressedSize(&self) -> c_uint {
        if self.core.is_null() {
            return 0;
        }

        match self.offsetBytes {
        1 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_uchar>)).CompressedSize()
        },
        2 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_ushort>)).CompressedSize()
        },
        3 => unsafe {
            (*(self.core as *mut SPARCCore<T, NotSoShort>)).CompressedSize()
        },
        4 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_uint>)).CompressedSize()
        },
        _ => 0,
        }
    }

    //Always use the same core type since we won't be compressing.
    pub fn Store(&mut self, array: *const T, length: c_uint) -> c_uint
    {
        self.Release();
        self.offsetBytes = 1;
        let core_box = Box::new(SPARCCore::<T, c_uchar>::new());
        let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_uchar>;
        unsafe {
            (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
        }
        self.core = core_raw as *mut c_void;
        unsafe {
            (*core_raw).Store(array, length)
        }
    }

    //Load compressed data directly.
    pub fn Load(&mut self, mut array: *const c_uchar, length: c_uint) -> c_uint {
        self.Release();

        unsafe {
            self.offsetBytes = *array;
            array = array.add(1);
        }

        match self.offsetBytes {
        1 => {
            let core_box = Box::new(SPARCCore::<T, c_uchar>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_uchar>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Load(array, length - 1)
            }
        },
        2 => {
            let core_box = Box::new(SPARCCore::<T, c_ushort>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_ushort>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Load(array, length - 1)
            }
        },
        3 => {
            let core_box = Box::new(SPARCCore::<T, NotSoShort>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, NotSoShort>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Load(array, length - 1)
            }
        },
        4 => {
            let core_box = Box::new(SPARCCore::<T, c_uint>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_uint>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Load(array, length - 1)
            }
        },
        _ => {
            debug_assert!(false);
            0
        },
        }
    }

    //Save compressed data into array.
    pub fn Save(&self, mut array: *mut c_uchar, length: c_uint, doSwap: bool) -> c_uint {
        unsafe {
            *array = self.offsetBytes;
            array = array.add(1);
        }

        match self.offsetBytes {
        1 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_uchar>)).Save(array, length - 1, doSwap)
        },
        2 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_ushort>)).Save(array, length - 1, doSwap)
        },
        3 => unsafe {
            (*(self.core as *mut SPARCCore<T, NotSoShort>)).Save(array, length - 1, doSwap)
        },
        4 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_uint>)).Save(array, length - 1, doSwap)
        },
        _ => {
            debug_assert!(false);
            0
        },
        }
    }

    //Create the smallest core possible for the given data.
    pub fn Compress(&mut self, array: *const T, length: c_uint, removal: T) -> c_uint {
        self.Release();

        if length < 256 {
            self.offsetBytes = 1;
            let core_box = Box::new(SPARCCore::<T, c_uchar>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_uchar>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Compress(array, length, removal)
            }
        } else if length < 65536 {
            self.offsetBytes = 2;
            let core_box = Box::new(SPARCCore::<T, c_ushort>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_ushort>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Compress(array, length, removal)
            }
        } else if length < 16777216 {
            self.offsetBytes = 3;
            let core_box = Box::new(SPARCCore::<T, NotSoShort>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, NotSoShort>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Compress(array, length, removal)
            }
        } else {
            self.offsetBytes = 4;
            let core_box = Box::new(SPARCCore::<T, c_uint>::new());
            let mut core_raw = Box::into_raw(core_box) as *mut SPARCCore<T, c_uint>;
            unsafe {
                (*core_raw).SetAllocator(self.Allocator, self.Deallocator);
                self.core = core_raw as *mut c_void;
                (*core_raw).Compress(array, length, removal)
            }
        }
    }

    //Cast to the correct core type and decompress.
    pub fn Decompress(&mut self, offset: c_uint, length: c_uint) -> *const T {
        if self.core.is_null() {
            return null_mut();
        }

        match self.offsetBytes {
        1 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_uchar>)).Decompress(offset, length)
        },
        2 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_ushort>)).Decompress(offset, length)
        },
        3 => unsafe {
            (*(self.core as *mut SPARCCore<T, NotSoShort>)).Decompress(offset, length)
        },
        4 => unsafe {
            (*(self.core as *mut SPARCCore<T, c_uint>)).Decompress(offset, length)
        },
        _ => null_mut(),
        }
    }

    //Destroy all compressed data and the current decompressed buffer.
    pub fn Release(&mut self) {
        if !self.core.is_null() {
            match self.offsetBytes {
            1 => unsafe {
                drop(Box::from_raw(self.core as *mut SPARCCore<T, c_uchar>));
            },
            2 => unsafe {
                drop(Box::from_raw(self.core as *mut SPARCCore<T, c_ushort>));
            },
            3 => unsafe {
                drop(Box::from_raw(self.core as *mut SPARCCore<T, NotSoShort>));
            },
            4 => unsafe {
                drop(Box::from_raw(self.core as *mut SPARCCore<T, c_uint>));
            },
            _ => {},
            }
            self.core = null_mut();
        }
    }
}

impl<T> Drop for SPARC<T>
where
    T: Clone + Copy + PartialEq + Default,
{
    fn drop(&mut self) {
        self.Release();
    }
}
