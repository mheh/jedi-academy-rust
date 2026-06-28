// BlockStream.h
// #pragma warning(disable : 4786)  //identifier was truncated
// #pragma warning(disable : 4514)  //unreffed inline func removed
// #pragma warning (push, 3)	//go back down to 3 for the stl include
// #pragma warning (pop)

use core::ffi::c_char;

// Stubs for external functions from engine
extern "C" {
    fn Z_Malloc(size: usize, tag: i32, clear: i32) -> *mut core::ffi::c_void;
    fn Z_Free(ptr: *mut core::ffi::c_void);
    fn ICARUS_Malloc(size: usize) -> *mut core::ffi::c_void;
    fn ICARUS_Free(ptr: *mut core::ffi::c_void);
}

// Note: FILE is a libc type; we keep it as opaque
use core::ffi::c_void;

// (I)nterpreted (B)lock (I)nstructions
pub const IBI_EXT: &[u8] = b".IBI";
pub const IBI_HEADER_ID: &[u8] = b"IBI";

pub const IBI_VERSION: f32 = 1.57f32;
pub const MAX_FILENAME_LENGTH: usize = 1024;

pub type vector_t = [f32; 3];

// Pop/push direction enum
pub const POP_FRONT: i32 = 0;
pub const POP_BACK: i32 = 1;
pub const PUSH_FRONT: i32 = 2;
pub const PUSH_BACK: i32 = 3;

// Templates

// CBlockMember

#[repr(C)]
pub struct CBlockMember {
    // ID of the value contained in data
    m_id: i32,
    // Size of the data member variable
    m_size: i32,
    // Data for this member
    m_data: *mut c_void,
}

impl CBlockMember {
    pub fn new() -> Self {
        CBlockMember {
            m_id: 0,
            m_size: 0,
            m_data: core::ptr::null_mut(),
        }
    }

    pub fn Free(&mut self) {
        // Implementation would be here
    }

    // Writes the member's data, in block format, to FILE *
    pub fn WriteMember(&self, _file: *mut c_void) -> i32 {
        0 // stub
    }

    // Reads the member's data, in block format, from FILE *
    pub fn ReadMember(&mut self, _data: *mut *mut c_char, _size: *mut i64) -> i32 {
        0 // stub
    }

    // Set the ID member variable
    pub fn SetID(&mut self, id: i32) {
        self.m_id = id;
    }

    // Set the size member variable
    pub fn SetSize(&mut self, size: i32) {
        self.m_size = size;
    }

    pub fn GetInfo(&self, _id: *mut i32, _size: *mut i32, _data: *mut *mut c_void) {
        // stub
    }

    // SetData overloads
    pub fn SetData_str(&mut self, data: *const c_char) {
        // stub
    }

    pub fn SetData_vector(&mut self, data: vector_t) {
        self.WriteData_impl(&data);
    }

    pub fn SetData_ptr(&mut self, data: *mut c_void, size: i32) {
        // stub
    }

    // Get ID member variables
    pub fn GetID(&self) -> i32 {
        self.m_id
    }

    // Get data member variable
    pub fn GetData(&self) -> *mut c_void {
        self.m_data
    }

    // Get size member variable
    pub fn GetSize(&self) -> i32 {
        self.m_size
    }

    pub fn Duplicate(&self) -> *mut CBlockMember {
        core::ptr::null_mut() // stub
    }

    // Template method: WriteData<T>
    // Translated as a generic method with buffer copying
    fn WriteData_impl<T: Sized>(&mut self, data: &T) {
        unsafe {
            if !self.m_data.is_null() {
                ICARUS_Free(self.m_data);
            }

            self.m_data = ICARUS_Malloc(core::mem::size_of::<T>());
            *(self.m_data as *mut T) = core::ptr::read(data);
            self.m_size = core::mem::size_of::<T>() as i32;
        }
    }

    // Template method: WriteDataPointer<T>
    // Translated as a method taking a pointer and count
    pub fn WriteDataPointer<T: Sized>(&mut self, data: *const T, num: i32) {
        unsafe {
            if !self.m_data.is_null() {
                ICARUS_Free(self.m_data);
            }

            let byte_size = (num as usize) * core::mem::size_of::<T>();
            self.m_data = ICARUS_Malloc(byte_size);
            core::ptr::copy_nonoverlapping(
                data as *const u8,
                self.m_data as *mut u8,
                byte_size,
            );
            self.m_size = byte_size as i32;
        }
    }
}

// CBlock

#[repr(C)]
pub struct CBlock {
    // List of all CBlockMembers owned by this list
    m_members: Vec<*mut CBlockMember>,
    // ID of the block
    m_id: i32,
    m_flags: u8,
}

impl CBlock {
    pub fn new() -> Self {
        CBlock {
            m_members: Vec::new(),
            m_id: 0,
            m_flags: 0,
        }
    }

    pub fn Init(&mut self) -> i32 {
        0 // stub
    }

    pub fn Create(&mut self, _id: i32) -> i32 {
        0 // stub
    }

    pub fn Free(&mut self) -> i32 {
        0 // stub
    }

    // Write Overloads
    pub fn Write_vector(&mut self, _id: i32, _data: vector_t) -> i32 {
        0 // stub
    }

    pub fn Write_float(&mut self, _id: i32, _data: f32) -> i32 {
        0 // stub
    }

    pub fn Write_str(&mut self, _id: i32, _data: *const c_char) -> i32 {
        0 // stub
    }

    pub fn Write_int(&mut self, _id: i32, _data: i32) -> i32 {
        0 // stub
    }

    pub fn Write_member(&mut self, _member: *mut CBlockMember) -> i32 {
        0 // stub
    }

    // Member push / pop functions
    pub fn AddMember(&mut self, member: *mut CBlockMember) -> i32 {
        self.m_members.push(member);
        0
    }

    pub fn GetMember(&self, memberNum: usize) -> *mut CBlockMember {
        if memberNum < self.m_members.len() {
            self.m_members[memberNum]
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn GetMemberData(&self, memberNum: usize) -> *mut c_void {
        if memberNum < self.m_members.len() {
            unsafe { (*self.m_members[memberNum]).GetData() }
        } else {
            core::ptr::null_mut()
        }
    }

    pub fn Duplicate(&self) -> *mut CBlock {
        core::ptr::null_mut() // stub
    }

    // Get the ID for the block
    pub fn GetBlockID(&self) -> i32 {
        self.m_id
    }

    // Get the number of member in the block's list
    pub fn GetNumMembers(&self) -> usize {
        self.m_members.len()
    }

    pub fn SetFlags(&mut self, flags: u8) {
        self.m_flags = flags;
    }

    pub fn SetFlag(&mut self, flag: u8) {
        self.m_flags |= flag;
    }

    pub fn HasFlag(&self, flag: u8) -> i32 {
        if (self.m_flags & flag) != 0 {
            1
        } else {
            0
        }
    }

    pub fn GetFlags(&self) -> u8 {
        self.m_flags
    }
}

// CBlockStream

#[repr(C)]
pub struct CBlockStream {
    // Size of the file
    m_fileSize: i64,
    // Global file handle of current I/O source
    m_fileHandle: *mut c_void,
    // Name of the current file
    m_fileName: [c_char; MAX_FILENAME_LENGTH],
    // Stream of data to be parsed
    m_stream: *mut c_char,
    m_streamPos: i64,
}

impl CBlockStream {
    pub fn new() -> Self {
        CBlockStream {
            m_fileSize: 0,
            m_fileHandle: core::ptr::null_mut(),
            m_fileName: [0; MAX_FILENAME_LENGTH],
            m_stream: core::ptr::null_mut(),
            m_streamPos: 0,
        }
    }

    pub fn Init(&mut self) -> i32 {
        0 // stub
    }

    pub fn Create(&mut self, _name: *mut c_char) -> i32 {
        0 // stub
    }

    pub fn Free(&mut self) -> i32 {
        0 // stub
    }

    // Stream I/O functions

    pub fn BlockAvailable(&self) -> i32 {
        0 // stub
    }

    // Write the block out
    pub fn WriteBlock(&mut self, _block: *mut CBlock) -> i32 {
        0 // stub
    }

    // Read the block in
    pub fn ReadBlock(&mut self, _block: *mut CBlock) -> i32 {
        0 // stub
    }

    // Open a stream for reading / writing
    pub fn Open(&mut self, _name: *mut c_char, _offset: i64) -> i32 {
        0 // stub
    }

    // Protected helper methods
    fn GetUnsignedInteger(&mut self) -> u32 {
        0 // stub
    }

    fn GetInteger(&mut self) -> i32 {
        0 // stub
    }

    fn GetChar(&mut self) -> c_char {
        0 // stub
    }

    fn GetLong(&mut self) -> i64 {
        0 // stub
    }

    fn GetFloat(&mut self) -> f32 {
        0.0 // stub
    }

    // Utility function to strip away file extensions
    fn StripExtension(&self, _in_str: *const c_char, _out_str: *mut c_char) {
        // stub
    }
}
