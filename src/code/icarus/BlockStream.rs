// Interpreted Block Stream Functions
//
//	-- jweier

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_mut,
    unused_assignments,
    unused_imports,
    clippy::all
)]

// this include must remain at the top of every Icarus CPP file
use crate::code::icarus::stdafx_h::*;

use crate::code::icarus::IcarusInterface_h::*;
use crate::code::icarus::IcarusImplementation_h::*;

use crate::code::icarus::BlockStream_h::*;

use core::ffi::{c_char, c_int, c_long, c_uchar, c_void};

// System/libc declarations (not translated modules; trusted C runtime symbols).
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dest: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strcat(dest: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut c_void; // FILE*
    fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize; // FILE*
}

/*
===================================================================================================

  CBlockMember

===================================================================================================
*/

impl CBlockMember {
    // inline CBlockMember::CBlockMember( void )
    pub fn new() -> Self {
        let mut member: CBlockMember = unsafe { core::mem::zeroed() };
        member.m_id = -1;
        member.m_size = -1;
        member.m_data = core::ptr::null_mut();
        member
    }

    // inline CBlockMember::~CBlockMember( void )
    // {
    // }
    // (empty destructor -- nothing to translate; kept as a comment only, matching the
    // porting convention used elsewhere in this module for trivial C++ destructors.)

    /*
    -------------------------
    Free
    -------------------------
    */

    // void CBlockMember::Free(IGameInterface* game)
    // `delete this;` at the end consumes the object -- translated as a free function
    // taking the raw pointer, matching the CTask::Free pattern used elsewhere in icarus/.
    pub unsafe fn Free(self_: *mut CBlockMember, game: *mut IGameInterface) {
        if !(*self_).m_data.is_null() {
            (*game).Free((*self_).m_data);
            (*self_).m_data = core::ptr::null_mut();

            (*self_).m_id = -1;
            (*self_).m_size = -1;
        }
        drop(Box::from_raw(self_));
    }

    /*
    -------------------------
    GetInfo
    -------------------------
    */

    pub unsafe fn GetInfo(&self, id: *mut c_int, size: *mut c_int, data: *mut *mut c_void) {
        *id = self.m_id;
        *size = self.m_size;
        *data = self.m_data;
    }

    /*
    -------------------------
    SetData overloads
    -------------------------
    */

    // void CBlockMember::SetData( const char *data , CIcarus* icarus)
    pub unsafe fn SetData_char(&mut self, data: *const c_char, icarus: *mut CIcarus) {
        self.WriteDataPointer(data, (strlen(data) + 1) as c_int, icarus);
    }

    // void CBlockMember::SetData( vec3_t data , CIcarus* icarus)
    // vec3_t decays to float* as a function parameter.
    pub unsafe fn SetData_vec3(&mut self, data: *mut f32, icarus: *mut CIcarus) {
        self.WriteDataPointer(data, 3, icarus);
    }

    // void CBlockMember::SetData( void *data, int size, CIcarus* icarus)
    pub unsafe fn SetData_void(&mut self, data: *mut c_void, size: c_int, icarus: *mut CIcarus) {
        let game: *mut IGameInterface = (*icarus).GetGame();
        if !self.m_data.is_null() {
            (*game).Free(self.m_data);
        }

        self.m_data = (*game).Malloc(size);
        memcpy(self.m_data, data, size as usize);
        self.m_size = size;
    }

    //	Member I/O functions

    /*
    -------------------------
    ReadMember
    -------------------------
    */

    pub unsafe fn ReadMember(
        &mut self,
        stream: *mut *mut c_char,
        streamPos: *mut c_long,
        icarus: *mut CIcarus,
    ) -> c_int {
        let game: *mut IGameInterface = (*icarus).GetGame();
        self.m_id = *((*stream).add(*streamPos as usize) as *mut c_int);
        *streamPos += core::mem::size_of::<c_int>() as c_long;

        if self.m_id == CIcarus::ID_RANDOM {
            //special case, need to initialize this member's data to Q3_INFINITE so we can randomize the number only the first time random is checked when inside a wait
            self.m_size = core::mem::size_of::<f32>() as c_int;
            *streamPos += core::mem::size_of::<c_long>() as c_long;
            self.m_data = (*game).Malloc(self.m_size);
            let infinite: f32 = (*game).MaxFloat();
            memcpy(self.m_data, &infinite as *const f32 as *const c_void, self.m_size as usize);
        } else {
            self.m_size = *((*stream).add(*streamPos as usize) as *mut c_long) as c_int;
            *streamPos += core::mem::size_of::<c_long>() as c_long;
            self.m_data = (*game).Malloc(self.m_size);
            memcpy(
                self.m_data,
                (*stream).add(*streamPos as usize) as *const c_void,
                self.m_size as usize,
            );
        }
        *streamPos += self.m_size as c_long;

        true as c_int
    }

    /*
    -------------------------
    WriteMember
    -------------------------
    */

    pub unsafe fn WriteMember(&self, m_fileHandle: *mut c_void) -> c_int {
        fwrite(
            &self.m_id as *const c_int as *const c_void,
            core::mem::size_of::<c_int>(),
            1,
            m_fileHandle,
        );
        fwrite(
            &self.m_size as *const c_int as *const c_void,
            core::mem::size_of::<c_int>(),
            1,
            m_fileHandle,
        );
        fwrite(self.m_data, self.m_size as usize, 1, m_fileHandle);

        true as c_int
    }

    /*
    -------------------------
    Duplicate
    -------------------------
    */

    pub unsafe fn Duplicate(&self, icarus: *mut CIcarus) -> *mut CBlockMember {
        let newblock: *mut CBlockMember = Box::into_raw(Box::new(CBlockMember::new()));

        if newblock.is_null() {
            return core::ptr::null_mut();
        }

        (*newblock).SetData_void(self.m_data, self.m_size, icarus);
        (*newblock).SetSize(self.m_size);
        (*newblock).SetID(self.m_id);

        newblock
    }
}

/*
===================================================================================================

  CBlock

===================================================================================================
*/


/*
-------------------------
Init
-------------------------
*/

impl CBlock {
    pub fn Init(&mut self) -> c_int {
        self.m_flags = 0;
        self.m_id = 0;

        true as c_int
    }

    /*
    -------------------------
    Create
    -------------------------
    */

    pub fn Create(&mut self, block_id: c_int) -> c_int {
        self.Init();

        self.m_id = block_id;

        true as c_int
    }

    /*
    -------------------------
    Free
    -------------------------
    */

    pub unsafe fn Free(&mut self, icarus: *mut CIcarus) -> c_int {
        let game: *mut IGameInterface = (*icarus).GetGame();
        let mut numMembers: c_int = self.GetNumMembers();
        let mut bMember: *mut CBlockMember;

        while numMembers > 0 {
            numMembers -= 1;
            bMember = self.GetMember(numMembers);

            if bMember.is_null() {
                return false as c_int;
            }

            CBlockMember::Free(bMember, game);
        }

        self.m_members.clear(); //List of all CBlockMembers owned by this list

        true as c_int
    }

    //	Write overloads

    /*
    -------------------------
    Write
    -------------------------
    */

    pub unsafe fn Write_char(&mut self, member_id: c_int, member_data: *const c_char, icarus: *mut CIcarus) -> c_int {
        let bMember: *mut CBlockMember = Box::into_raw(Box::new(CBlockMember::new()));

        (*bMember).SetID(member_id);

        (*bMember).SetData_char(member_data, icarus);
        (*bMember).SetSize((strlen(member_data) + 1) as c_int);

        self.AddMember(bMember);

        true as c_int
    }

    // vec3_t decays to float* as a function parameter.
    pub unsafe fn Write_vec3(&mut self, member_id: c_int, member_data: *mut f32, icarus: *mut CIcarus) -> c_int {
        let bMember: *mut CBlockMember;

        bMember = Box::into_raw(Box::new(CBlockMember::new()));

        (*bMember).SetID(member_id);
        (*bMember).SetData_vec3(member_data, icarus);
        (*bMember).SetSize(core::mem::size_of::<vec3_t>() as c_int);

        self.AddMember(bMember);

        true as c_int
    }

    pub unsafe fn Write_float(&mut self, member_id: c_int, member_data: f32, icarus: *mut CIcarus) -> c_int {
        let bMember: *mut CBlockMember = Box::into_raw(Box::new(CBlockMember::new()));

        (*bMember).SetID(member_id);
        (*bMember).WriteData(member_data, icarus);
        (*bMember).SetSize(core::mem::size_of_val(&member_data) as c_int);

        self.AddMember(bMember);

        true as c_int
    }

    pub unsafe fn Write_int(&mut self, member_id: c_int, member_data: c_int, icarus: *mut CIcarus) -> c_int {
        let bMember: *mut CBlockMember = Box::into_raw(Box::new(CBlockMember::new()));

        (*bMember).SetID(member_id);
        (*bMember).WriteData(member_data, icarus);
        (*bMember).SetSize(core::mem::size_of_val(&member_data) as c_int);

        self.AddMember(bMember);

        true as c_int
    }

    pub unsafe fn Write_member(&mut self, bMember: *mut CBlockMember, _icarus: *mut CIcarus) -> c_int {
        // findme: this is wrong:	bMember->SetSize( sizeof(bMember->GetData()) );

        self.AddMember(bMember);

        true as c_int
    }

    // Member list functions

    /*
    -------------------------
    AddMember
    -------------------------
    */

    pub fn AddMember(&mut self, member: *mut CBlockMember) -> c_int {
        self.m_members.insert(self.m_members.len(), member);
        true as c_int
    }

    /*
    -------------------------
    GetMember
    -------------------------
    */

    pub fn GetMember(&self, memberNum: c_int) -> *mut CBlockMember {
        if memberNum > self.GetNumMembers() - 1 {
            return core::ptr::null_mut(); // return false;
        }
        self.m_members[memberNum as usize]
    }

    /*
    -------------------------
    GetMemberData
    -------------------------
    */

    pub unsafe fn GetMemberData(&self, memberNum: c_int) -> *mut c_void {
        if memberNum > self.GetNumMembers() - 1 {
            return core::ptr::null_mut();
        }
        (*self.GetMember(memberNum)).GetData() as *mut c_void
    }

    /*
    -------------------------
    Duplicate
    -------------------------
    */

    pub unsafe fn Duplicate(&self, icarus: *mut CIcarus) -> *mut CBlock {
        let newblock: *mut CBlock;

        newblock = Box::into_raw(Box::new(CBlock::new()));

        if newblock.is_null() {
            return core::ptr::null_mut(); // return false;
        }

        (*newblock).Create(self.m_id);

        //Duplicate entire block and return the cc
        for mi in self.m_members.iter() {
            (*newblock).AddMember((**mi).Duplicate(icarus));
        }

        newblock
    }
}

/*
===================================================================================================

  CBlockStream

===================================================================================================
*/

// char* CBlockStream::s_IBI_EXT				= ".IBI";	//(I)nterpreted (B)lock (I)nstructions
// char* CBlockStream::s_IBI_HEADER_ID			= "IBI";
// const float	CBlockStream::s_IBI_VERSION		= 1.57f;
//
// These are the storage definitions for the static class members declared in BlockStream.h;
// this .cpp file is where the linker-visible storage actually lives (the header only declares
// them), so they are defined here rather than imported.
pub static mut s_IBI_EXT: *mut c_char = b".IBI\0".as_ptr() as *mut c_char; //(I)nterpreted (B)lock (I)nstructions
pub static mut s_IBI_HEADER_ID: *mut c_char = b"IBI\0".as_ptr() as *mut c_char;
pub static s_IBI_VERSION: f32 = 1.57f32;

impl CBlockStream {
    /*
    -------------------------
    Free
    -------------------------
    */

    pub fn Free(&mut self) -> c_int {
        //NOTENOTE: It is assumed that the user will free the passed memory block (m_stream) immediately after the run call
        //			That's why this doesn't free the memory, it only clears its internal pointer

        self.m_stream = core::ptr::null_mut();
        self.m_streamPos = 0;

        true as c_int
    }

    /*
    -------------------------
    Create
    -------------------------
    */

    pub unsafe fn Create(&mut self, filename: *mut c_char) -> c_int {
        // strip extension
        let mut extensionloc: c_int = strlen(filename) as c_int;
        while (*filename.offset(extensionloc as isize) != b'.' as c_char) && (extensionloc >= 0) {
            extensionloc -= 1;
        }
        if extensionloc < 0 {
            strcpy(self.m_fileName.as_mut_ptr(), filename);
        } else {
            strncpy(self.m_fileName.as_mut_ptr(), filename, extensionloc as usize);
            *self.m_fileName.as_mut_ptr().add(extensionloc as usize) = 0;
        }
        // add extension
        strcat(self.m_fileName.as_mut_ptr() as *mut c_char, s_IBI_EXT as *const c_char);

        if { self.m_fileHandle = fopen(self.m_fileName.as_ptr(), "wb\0".as_ptr() as *const c_char); self.m_fileHandle.is_null() } {
            return false as c_int;
        }

        fwrite(
            s_IBI_HEADER_ID as *const c_void,
            1,
            core::mem::size_of::<*mut c_char>(),
            self.m_fileHandle,
        );
        fwrite(
            &s_IBI_VERSION as *const f32 as *const c_void,
            1,
            core::mem::size_of::<f32>(),
            self.m_fileHandle,
        );

        true as c_int
    }

    /*
    -------------------------
    Init
    -------------------------
    */

    pub unsafe fn Init(&mut self) -> c_int {
        self.m_fileHandle = core::ptr::null_mut();
        memset(
            self.m_fileName.as_mut_ptr() as *mut c_void,
            0,
            core::mem::size_of_val(&self.m_fileName),
        );

        self.m_stream = core::ptr::null_mut();
        self.m_streamPos = 0;

        true as c_int
    }

    //	Block I/O functions

    /*
    -------------------------
    WriteBlock
    -------------------------
    */

    pub unsafe fn WriteBlock(&mut self, block: *mut CBlock, icarus: *mut CIcarus) -> c_int {
        let mut bMember: *mut CBlockMember;
        let id: c_int = (*block).GetBlockID();
        let numMembers: c_int = (*block).GetNumMembers();
        let flags: c_uchar = (*block).GetFlags();

        fwrite(&id as *const c_int as *const c_void, core::mem::size_of::<c_int>(), 1, self.m_fileHandle);
        fwrite(
            &numMembers as *const c_int as *const c_void,
            core::mem::size_of::<c_int>(),
            1,
            self.m_fileHandle,
        );
        fwrite(
            &flags as *const c_uchar as *const c_void,
            core::mem::size_of::<c_uchar>(),
            1,
            self.m_fileHandle,
        );

        let mut i: c_int = 0;
        while i < numMembers {
            bMember = (*block).GetMember(i);
            (*bMember).WriteMember(self.m_fileHandle);
            i += 1;
        }

        (*block).Free(icarus);

        true as c_int
    }

    /*
    -------------------------
    BlockAvailable
    -------------------------
    */

    pub fn BlockAvailable(&self) -> c_int {
        if self.m_streamPos >= self.m_fileSize {
            return false as c_int;
        }

        true as c_int
    }

    /*
    -------------------------
    ReadBlock
    -------------------------
    */

    pub unsafe fn ReadBlock(&mut self, get: *mut CBlock, icarus: *mut CIcarus) -> c_int {
        let mut bMember: *mut CBlockMember;
        let b_id: c_int;
        let mut numMembers: c_int;
        let flags: c_uchar;

        if self.BlockAvailable() == false as c_int {
            return false as c_int;
        }

        b_id = *(self.m_stream.add(self.m_streamPos as usize) as *mut c_int);
        self.m_streamPos += core::mem::size_of::<c_int>() as c_long;

        numMembers = *(self.m_stream.add(self.m_streamPos as usize) as *mut c_int);
        self.m_streamPos += core::mem::size_of::<c_int>() as c_long;

        flags = *(self.m_stream.add(self.m_streamPos as usize) as *mut c_uchar);
        self.m_streamPos += core::mem::size_of::<c_uchar>() as c_long;

        if numMembers < 0 {
            return false as c_int;
        }

        (*get).Create(b_id);
        (*get).SetFlags(flags);

        // Stream blocks are generally temporary as they
        // are just used in an initial parsing phase...
        #[cfg(feature = "xbox")]
        {
            extern "C" {
                fn Z_SetNewDeleteTemporary(bTemp: bool);
            }
            Z_SetNewDeleteTemporary(true);
        }

        while numMembers > 0 {
            numMembers -= 1;
            bMember = Box::into_raw(Box::new(CBlockMember::new()));
            (*bMember).ReadMember(&mut self.m_stream, &mut self.m_streamPos, icarus);
            (*get).AddMember(bMember);
        }

        #[cfg(feature = "xbox")]
        {
            extern "C" {
                fn Z_SetNewDeleteTemporary(bTemp: bool);
            }
            Z_SetNewDeleteTemporary(false);
        }

        true as c_int
    }

    /*
    -------------------------
    Open
    -------------------------
    */

    pub unsafe fn Open(&mut self, buffer: *mut c_char, size: c_long) -> c_int {
        let mut id_header: [c_char; core::mem::size_of::<*mut c_char>()] =
            [0; core::mem::size_of::<*mut c_char>()];
        let version: f32;

        self.Init();

        self.m_fileSize = size;

        self.m_stream = buffer;

        let mut i: usize = 0;
        while i < core::mem::size_of_val(&id_header) {
            id_header[i] = *self.m_stream.add(self.m_streamPos as usize);
            self.m_streamPos += 1;
            i += 1;
        }

        version = *(self.m_stream.add(self.m_streamPos as usize) as *mut f32);
        self.m_streamPos += core::mem::size_of::<f32>() as c_long;

        //Check for valid header
        if strcmp(id_header.as_ptr(), s_IBI_HEADER_ID as *const c_char) != 0 {
            self.Free();
            return false as c_int;
        }

        //Check for valid version
        if version != s_IBI_VERSION {
            self.Free();
            return false as c_int;
        }

        true as c_int
    }
}
