// Anything above this #include will be ignored by the compiler
#include "../qcommon/exe_headers.h"

// Interpreted Block Stream Functions
//
//	-- jweier

// this include must remain at the top of every Icarus CPP file
#include "icarus.h"

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};
use std::ptr::{addr_of, addr_of_mut, null, null_mut};

use super::blockstream_h::{
    CBlockMember, CBlock, CBlockStream, IBI_EXT, IBI_HEADER_ID, IBI_VERSION,
    MAX_FILENAME_LENGTH, vector_t,
};

// Libc and engine function declarations
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fclose(stream: *mut c_void) -> c_int;
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
}

// From Memory.rs
extern "C" {
    fn ICARUS_Malloc(size: c_int) -> *mut c_void;
    fn ICARUS_Free(ptr: *mut c_void);
}

// Constants
const Q3_INFINITE: f32 = 16777216.0f32;
const ID_RANDOM: i32 = 5;

// ===================================================================================================
//
//  CBlockMember
//
// ===================================================================================================

impl CBlockMember {
    // CBlockMember constructor
    pub fn new() -> Self {
        CBlockMember {
            m_id: -1,
            m_size: -1,
            m_data: null_mut(),
        }
    }

    // CBlockMember destructor
    pub fn drop(&mut self) {
        self.Free();
    }

    // -------------------------
    // Free
    // -------------------------

    pub fn Free(&mut self) {
        unsafe {
            if !self.m_data.is_null() {
                ICARUS_Free(self.m_data);
                self.m_data = null_mut();

                self.m_id = -1;
                self.m_size = -1;
            }
        }
    }

    // -------------------------
    // GetInfo
    // -------------------------

    pub fn GetInfo(&self, id: *mut i32, size: *mut i32, data: *mut *mut c_void) {
        unsafe {
            *id = self.m_id;
            *size = self.m_size;
            *data = self.m_data;
        }
    }

    // -------------------------
    // SetData overloads
    // -------------------------

    pub fn SetData_str(&mut self, data: *const c_char) {
        unsafe {
            let len = strlen(data) + 1;
            self.WriteDataPointer(data, len as i32);
        }
    }

    pub fn SetData_vector(&mut self, data: vector_t) {
        self.WriteDataPointer(addr_of!(data) as *const c_char, 3 as i32);
    }

    pub fn SetData(&mut self, data: *mut c_void, size: i32) {
        unsafe {
            if !self.m_data.is_null() {
                ICARUS_Free(self.m_data);
            }

            self.m_data = ICARUS_Malloc(size);
            memcpy(self.m_data, data, size as usize);
            self.m_size = size;
        }
    }

    // -------------------------
    // Member I/O functions
    // -------------------------

    // -------------------------
    // ReadMember
    // -------------------------

    pub fn ReadMember(&mut self, stream: *mut *mut c_char, streamPos: *mut i64) -> i32 {
        unsafe {
            self.m_id = *((*stream as *const i32).add(*streamPos as usize / std::mem::size_of::<i32>())) as i32;
            *streamPos += std::mem::size_of::<i32>() as i64;

            if self.m_id == ID_RANDOM {
                // special case, need to initialize this member's data to Q3_INFINITE so we can randomize the number only the first time random is checked when inside a wait
                self.m_size = std::mem::size_of::<f32>() as i32;
                *streamPos += std::mem::size_of::<i64>() as i64;
                self.m_data = ICARUS_Malloc(self.m_size);
                let infinite = Q3_INFINITE;
                memcpy(
                    self.m_data,
                    addr_of!(infinite) as *const c_void,
                    self.m_size as usize,
                );
            } else {
                self.m_size = *((*stream as *const i64).add(*streamPos as usize / std::mem::size_of::<i64>())) as i32;
                *streamPos += std::mem::size_of::<i64>() as i64;
                self.m_data = ICARUS_Malloc(self.m_size);
                memcpy(
                    self.m_data,
                    (*stream as *const c_void).add(*streamPos as usize),
                    self.m_size as usize,
                );
            }
            *streamPos += self.m_size as i64;

            return 1; // true
        }
    }

    // -------------------------
    // WriteMember
    // -------------------------

    pub fn WriteMember(&self, m_fileHandle: *mut c_void) -> i32 {
        unsafe {
            fwrite(
                addr_of!(self.m_id) as *const c_void,
                std::mem::size_of::<i32>(),
                1,
                m_fileHandle,
            );
            fwrite(
                addr_of!(self.m_size) as *const c_void,
                std::mem::size_of::<i32>(),
                1,
                m_fileHandle,
            );
            fwrite(self.m_data, self.m_size as usize, 1, m_fileHandle);

            return 1; // true
        }
    }

    // -------------------------
    // Duplicate
    // -------------------------

    pub fn Duplicate(&self) -> *mut CBlockMember {
        unsafe {
            let newblock = Box::into_raw(Box::new(CBlockMember::new()));

            if newblock.is_null() {
                return null_mut();
            }

            (*newblock).SetData(self.m_data, self.m_size);
            (*newblock).SetSize(self.m_size);
            (*newblock).SetID(self.m_id);

            return newblock;
        }
    }

    // Template method: WriteData<T>
    pub fn WriteData<T: Sized>(&mut self, data: &T) {
        unsafe {
            if !self.m_data.is_null() {
                ICARUS_Free(self.m_data);
            }

            self.m_data = ICARUS_Malloc(std::mem::size_of::<T>() as c_int);
            *(self.m_data as *mut T) = core::ptr::read(data);
            self.m_size = std::mem::size_of::<T>() as i32;
        }
    }

    // Template method: WriteDataPointer<T>
    pub fn WriteDataPointer(&mut self, data: *const c_char, num: i32) {
        unsafe {
            if !self.m_data.is_null() {
                ICARUS_Free(self.m_data);
            }

            let byte_size = (num as usize) * std::mem::size_of::<c_char>();
            self.m_data = ICARUS_Malloc(byte_size as c_int);
            memcpy(self.m_data, data as *const c_void, byte_size);
            self.m_size = byte_size as i32;
        }
    }

    // Inline getters/setters from header
    pub fn SetID(&mut self, id: i32) {
        self.m_id = id;
    }

    pub fn SetSize(&mut self, size: i32) {
        self.m_size = size;
    }

    pub fn GetID(&self) -> i32 {
        self.m_id
    }

    pub fn GetData(&self) -> *mut c_void {
        self.m_data
    }

    pub fn GetSize(&self) -> i32 {
        self.m_size
    }
}

// ===================================================================================================
//
//  CBlock
//
// ===================================================================================================

impl CBlock {
    // CBlock constructor
    pub fn new() -> Self {
        CBlock {
            m_flags: 0,
            m_id: 0,
            m_members: Vec::new(),
        }
    }

    // -------------------------
    // Init
    // -------------------------

    pub fn Init(&mut self) -> i32 {
        self.m_flags = 0;
        self.m_id = 0;

        return 1; // true
    }

    // -------------------------
    // Create
    // -------------------------

    pub fn Create(&mut self, block_id: i32) -> i32 {
        self.Init();

        self.m_id = block_id;

        return 1; // true
    }

    // -------------------------
    // Free
    // -------------------------

    pub fn Free(&mut self) -> i32 {
        let numMembers = self.GetNumMembers();

        for i in (0..numMembers).rev() {
            let bMember = self.GetMember(i);

            if bMember.is_null() {
                return 0; // false
            }

            unsafe {
                let _ = Box::from_raw(bMember);
            }
        }

        self.m_members.clear();

        return 1; // true
    }

    // -------------------------
    // Write overloads
    // -------------------------

    pub fn Write_str(&mut self, member_id: i32, member_data: *const c_char) -> i32 {
        unsafe {
            let bMember = Box::into_raw(Box::new(CBlockMember::new()));

            (*bMember).SetID(member_id);

            (*bMember).SetData_str(member_data);
            (*bMember).SetSize((strlen(member_data) + 1) as i32);

            self.AddMember(bMember);

            return 1; // true
        }
    }

    pub fn Write_vector(&mut self, member_id: i32, member_data: vector_t) -> i32 {
        unsafe {
            let bMember = Box::into_raw(Box::new(CBlockMember::new()));

            (*bMember).SetID(member_id);
            (*bMember).SetData_vector(member_data);
            (*bMember).SetSize(std::mem::size_of::<vector_t>() as i32);

            self.AddMember(bMember);

            return 1; // true
        }
    }

    pub fn Write_float(&mut self, member_id: i32, member_data: f32) -> i32 {
        unsafe {
            let bMember = Box::into_raw(Box::new(CBlockMember::new()));

            (*bMember).SetID(member_id);
            (*bMember).WriteData(&member_data);
            (*bMember).SetSize(std::mem::size_of::<f32>() as i32);

            self.AddMember(bMember);

            return 1; // true
        }
    }

    pub fn Write_int(&mut self, member_id: i32, member_data: i32) -> i32 {
        unsafe {
            let bMember = Box::into_raw(Box::new(CBlockMember::new()));

            (*bMember).SetID(member_id);
            (*bMember).WriteData(&member_data);
            (*bMember).SetSize(std::mem::size_of::<i32>() as i32);

            self.AddMember(bMember);

            return 1; // true
        }
    }

    pub fn Write_member(&mut self, bMember: *mut CBlockMember) -> i32 {
        // findme: this is wrong:	bMember->SetSize( sizeof(bMember->GetData()) );

        self.AddMember(bMember);

        return 1; // true
    }

    // -------------------------
    // AddMember
    // -------------------------

    pub fn AddMember(&mut self, member: *mut CBlockMember) -> i32 {
        self.m_members.push(member);
        return 1; // true
    }

    // -------------------------
    // GetMember
    // -------------------------

    pub fn GetMember(&self, memberNum: usize) -> *mut CBlockMember {
        if memberNum > self.GetNumMembers() - 1 {
            return null_mut(); // false
        }
        return self.m_members[memberNum];
    }

    // -------------------------
    // GetMemberData
    // -------------------------

    pub fn GetMemberData(&self, memberNum: usize) -> *mut c_void {
        if memberNum > self.GetNumMembers() - 1 {
            return null_mut();
        }
        unsafe {
            return ((*self.GetMember(memberNum)).GetData()) as *mut c_void;
        }
    }

    // -------------------------
    // Duplicate
    // -------------------------

    pub fn Duplicate(&self) -> *mut CBlock {
        unsafe {
            let newblock = Box::into_raw(Box::new(CBlock::new()));

            if newblock.is_null() {
                return null_mut(); // false
            }

            (*newblock).Create(self.m_id);

            // Duplicate entire block and return the cc
            for mi in self.m_members.iter() {
                (*newblock).AddMember((**mi).Duplicate());
            }

            return newblock;
        }
    }

    // Inline getters/setters from header
    pub fn GetBlockID(&self) -> i32 {
        self.m_id
    }

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

// ===================================================================================================
//
//  CBlockStream
//
// ===================================================================================================

impl CBlockStream {
    // CBlockStream constructor
    pub fn new() -> Self {
        CBlockStream {
            m_stream: null_mut(),
            m_streamPos: 0,
            m_fileSize: 0,
            m_fileHandle: null_mut(),
            m_fileName: [0; MAX_FILENAME_LENGTH],
        }
    }

    // -------------------------
    // GetChar
    // -------------------------

    pub fn GetChar(&mut self) -> c_char {
        unsafe {
            let data = *(self.m_stream.add(self.m_streamPos as usize)) as c_char;
            self.m_streamPos += std::mem::size_of::<c_char>() as i64;

            return data;
        }
    }

    // -------------------------
    // GetUnsignedInteger
    // -------------------------

    pub fn GetUnsignedInteger(&mut self) -> u32 {
        unsafe {
            let data = *((self.m_stream as *const u32).add(self.m_streamPos as usize / std::mem::size_of::<u32>()));
            self.m_streamPos += std::mem::size_of::<u32>() as i64;

            return data;
        }
    }

    // -------------------------
    // GetInteger
    // -------------------------

    pub fn GetInteger(&mut self) -> i32 {
        unsafe {
            let data = *((self.m_stream as *const i32).add(self.m_streamPos as usize / std::mem::size_of::<i32>()));
            self.m_streamPos += std::mem::size_of::<i32>() as i64;

            return data;
        }
    }

    // -------------------------
    // GetLong
    // -------------------------

    pub fn GetLong(&mut self) -> i64 {
        unsafe {
            let data = *((self.m_stream as *const i64).add(self.m_streamPos as usize / std::mem::size_of::<i64>()));
            self.m_streamPos += std::mem::size_of::<i64>() as i64;

            return data;
        }
    }

    // -------------------------
    // GetFloat
    // -------------------------

    pub fn GetFloat(&mut self) -> f32 {
        unsafe {
            let data = *((self.m_stream as *const f32).add(self.m_streamPos as usize / std::mem::size_of::<f32>()));
            self.m_streamPos += std::mem::size_of::<f32>() as i64;

            return data;
        }
    }

    // -------------------------
    // StripExtension
    // -------------------------

    pub fn StripExtension(&self, r#in: *const c_char, out: *mut c_char) {
        unsafe {
            let mut i = strlen(r#in) as i32;

            while (r#in.add(i as usize).read()) != ('.' as c_char) && (i >= 0) {
                i -= 1;
            }

            if i < 0 {
                strcpy(out, r#in);
                return;
            }

            strncpy(out, r#in, i as usize);
        }
    }

    // -------------------------
    // Free
    // -------------------------

    pub fn Free(&mut self) -> i32 {
        // NOTENOTE: It is assumed that the user will free the passed memory block (m_stream) immediately after the run call
        //			That's why this doesn't free the memory, it only clears its internal pointer

        self.m_stream = null_mut();
        self.m_streamPos = 0;

        return 1; // true
    }

    // -------------------------
    // Create
    // -------------------------

    pub fn Create(&mut self, filename: *mut c_char) -> i32 {
        unsafe {
            let mut newName: [c_char; MAX_FILENAME_LENGTH] = [0; MAX_FILENAME_LENGTH];
            let id_header = IBI_HEADER_ID;
            let version = IBI_VERSION;

            // Clear the temp string
            memset(
                newName.as_mut_ptr() as *mut c_void,
                0,
                std::mem::size_of::<[c_char; MAX_FILENAME_LENGTH]>(),
            );

            // Strip the extension and add the BLOCK_EXT extension
            strcpy(self.m_fileName.as_mut_ptr(), filename);
            self.StripExtension(self.m_fileName.as_ptr(), newName.as_mut_ptr());
            strcat(newName.as_mut_ptr(), IBI_EXT.as_ptr() as *const c_char);

            // Recover that as the active filename
            strcpy(
                self.m_fileName.as_mut_ptr(),
                newName.as_ptr(),
            );

            self.m_fileHandle = fopen(
                self.m_fileName.as_ptr(),
                "wb\0".as_ptr() as *const c_char,
            );

            if self.m_fileHandle.is_null() {
                return 0; // false
            }

            fwrite(
                id_header.as_ptr() as *const c_void,
                1,
                id_header.len(),
                self.m_fileHandle,
            );
            fwrite(
                addr_of!(version) as *const c_void,
                1,
                std::mem::size_of::<f32>(),
                self.m_fileHandle,
            );

            return 1; // true
        }
    }

    // -------------------------
    // Init
    // -------------------------

    pub fn Init(&mut self) -> i32 {
        self.m_fileHandle = null_mut();
        unsafe {
            memset(
                self.m_fileName.as_mut_ptr() as *mut c_void,
                0,
                std::mem::size_of::<[c_char; MAX_FILENAME_LENGTH]>(),
            );
        }

        self.m_stream = null_mut();
        self.m_streamPos = 0;

        return 1; // true
    }

    // -------------------------
    // WriteBlock
    // -------------------------

    pub fn WriteBlock(&mut self, block: *mut CBlock) -> i32 {
        unsafe {
            let id = (*block).GetBlockID();
            let numMembers = (*block).GetNumMembers();
            let flags = (*block).GetFlags();

            fwrite(
                addr_of!(id as i32) as *const c_void,
                std::mem::size_of::<i32>(),
                1,
                self.m_fileHandle,
            );
            fwrite(
                addr_of!(numMembers as i32) as *const c_void,
                std::mem::size_of::<i32>(),
                1,
                self.m_fileHandle,
            );
            fwrite(
                addr_of!(flags) as *const c_void,
                std::mem::size_of::<u8>(),
                1,
                self.m_fileHandle,
            );

            for i in 0..numMembers {
                let bMember = (*block).GetMember(i);
                (*bMember).WriteMember(self.m_fileHandle);
            }

            (*block).Free();

            return 1; // true
        }
    }

    // -------------------------
    // BlockAvailable
    // -------------------------

    pub fn BlockAvailable(&self) -> i32 {
        if self.m_streamPos >= self.m_fileSize {
            return 0; // false
        }

        return 1; // true
    }

    // -------------------------
    // ReadBlock
    // -------------------------

    pub fn ReadBlock(&mut self, get: *mut CBlock) -> i32 {
        unsafe {
            if self.BlockAvailable() == 0 {
                return 0; // false
            }

            let b_id = self.GetInteger();
            let numMembers = self.GetInteger();
            let flags = self.GetChar() as u8;

            if numMembers < 0 {
                return 0; // false
            }

            (*get).Create(b_id);
            (*get).SetFlags(flags);

            // Stream blocks are generally temporary as they
            // are just used in an initial parsing phase...
            #[cfg(target_os = "xbox")]
            {
                extern "C" {
                    fn Z_SetNewDeleteTemporary(bTemp: i32);
                }
                Z_SetNewDeleteTemporary(1);
            }

            let mut remaining = numMembers;
            while remaining > 0 {
                remaining -= 1;
                let bMember = Box::into_raw(Box::new(CBlockMember::new()));
                (*bMember).ReadMember(
                    addr_of_mut!(self.m_stream),
                    addr_of_mut!(self.m_streamPos),
                );
                (*get).AddMember(bMember);
            }

            #[cfg(target_os = "xbox")]
            {
                extern "C" {
                    fn Z_SetNewDeleteTemporary(bTemp: i32);
                }
                Z_SetNewDeleteTemporary(0);
            }

            return 1; // true
        }
    }

    // -------------------------
    // Open
    // -------------------------

    pub fn Open(&mut self, buffer: *mut c_char, size: i64) -> i32 {
        unsafe {
            let mut id_header: [c_char; 3] = [0; 3];
            let mut version: f32 = 0.0;

            self.Init();

            self.m_fileSize = size;

            self.m_stream = buffer;

            for i in 0..id_header.len() {
                id_header[i] = self.GetChar();
            }

            version = self.GetFloat();

            // Check for valid header
            if strcmp(id_header.as_ptr(), IBI_HEADER_ID.as_ptr() as *const c_char) != 0 {
                self.Free();
                return 0; // false
            }

            // Check for valid version
            if version != IBI_VERSION {
                self.Free();
                return 0; // false
            }

            return 1; // true
        }
    }
}
