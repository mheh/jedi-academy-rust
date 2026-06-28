// Interpreted Block Stream Functions
//
//	-- jweier

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_uchar, c_void};
use std::ptr;

// External types (opaque stubs)
#[repr(C)]
pub struct IGameInterface {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CIcarus {
    _private: [u8; 0],
}

// Type aliases to match C definitions
pub type vec3_t = [f32; 3];
pub type blockMember_v = Vec<*mut CBlockMember>;

// External C library functions
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    pub fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn fopen(filename: *const c_char, mode: *const c_char) -> *mut c_void;
    pub fn fwrite(ptr: *const c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    pub fn fread(ptr: *mut c_void, size: usize, nmemb: usize, stream: *mut c_void) -> usize;
    pub fn fclose(stream: *mut c_void) -> c_int;
}

// External game interface functions - these are virtual methods in C++
// In a faithful port, we declare them as functions to be linked from external code
extern "C" {
    // These are declared as function pointers or external implementations
    // that will be provided by the game engine
    pub fn game_interface_free(game: *mut IGameInterface, ptr: *mut c_void);
    pub fn game_interface_malloc(game: *mut IGameInterface, size: usize) -> *mut c_void;
    pub fn game_interface_maxfloat(game: *mut IGameInterface) -> f32;
    pub fn icarus_get_game(icarus: *mut CIcarus) -> *mut IGameInterface;
}

/*
===================================================================================================

  CBlockMember

===================================================================================================
*/

#[repr(C)]
pub struct CBlockMember {
    m_id: c_int,
    m_size: c_int,
    m_data: *mut c_void,
}

impl CBlockMember {
    // Inline constructor equivalent
    pub fn new() -> Box<CBlockMember> {
        Box::new(CBlockMember {
            m_id: -1,
            m_size: -1,
            m_data: ptr::null_mut(),
        })
    }

    // Inline destructor equivalent (handled by Rust's Drop trait)
    pub fn new_boxed() -> Box<CBlockMember> {
        Self::new()
    }

    /*
    -------------------------
    Free
    -------------------------
    */

    pub unsafe fn Free(&mut self, game: *mut IGameInterface) {
        if !self.m_data.is_null() {
            // game->Free ( m_data );
            game_interface_free(game, self.m_data);
            self.m_data = ptr::null_mut();

            self.m_id = -1;
            self.m_size = -1;
        }
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

    pub unsafe fn SetData_char(&mut self, data: *const c_char, icarus: *mut CIcarus) {
        let len = strlen(data) + 1;
        self.WriteDataPointer(data as *const c_void, len, icarus);
    }

    pub unsafe fn SetData_vec3(&mut self, data: *const vec3_t, icarus: *mut CIcarus) {
        self.WriteDataPointer(data as *const c_void, 3, icarus);
    }

    pub unsafe fn SetData_void(&mut self, data: *mut c_void, size: usize, icarus: *mut CIcarus) {
        let game = icarus_get_game(icarus);
        if !self.m_data.is_null() {
            game_interface_free(game, self.m_data);
        }

        self.m_data = game_interface_malloc(game, size);
        memcpy(self.m_data, data, size);
        self.m_size = size as c_int;
    }

    // Helper functions
    pub fn SetID(&mut self, id: c_int) {
        self.m_id = id;
    }

    pub fn SetSize(&mut self, size: c_int) {
        self.m_size = size;
    }

    pub fn GetData(&self) -> *mut c_void {
        self.m_data
    }

    pub fn GetID(&self) -> c_int {
        self.m_id
    }

    pub unsafe fn WriteDataPointer(&mut self, data: *const c_void, size: usize, icarus: *mut CIcarus) {
        let game = icarus_get_game(icarus);
        if !self.m_data.is_null() {
            game_interface_free(game, self.m_data);
        }

        self.m_data = game_interface_malloc(game, size);
        memcpy(self.m_data, data, size);
        self.m_size = size as c_int;
    }

    pub unsafe fn WriteData(&mut self, data: c_int, icarus: *mut CIcarus) {
        self.WriteDataPointer(&data as *const _ as *const c_void, core::mem::size_of::<c_int>(), icarus);
    }

    pub unsafe fn WriteData_float(&mut self, data: f32, icarus: *mut CIcarus) {
        self.WriteDataPointer(&data as *const _ as *const c_void, core::mem::size_of::<f32>(), icarus);
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
        let game = icarus_get_game(icarus);
        self.m_id = *((*stream as *mut c_int).add(*streamPos as usize / core::mem::size_of::<c_int>()));
        *streamPos += core::mem::size_of::<c_int>() as c_long;

        if self.m_id == CIcarus_ID_RANDOM {
            //special case, need to initialize this member's data to Q3_INFINITE so we can randomize the number only the first time random is checked when inside a wait
            self.m_size = core::mem::size_of::<f32>() as c_int;
            *streamPos += core::mem::size_of::<c_long>() as c_long;
            self.m_data = game_interface_malloc(game, self.m_size as usize);
            let infinite = game_interface_maxfloat(game);
            memcpy(
                self.m_data,
                &infinite as *const _ as *const c_void,
                core::mem::size_of::<f32>(),
            );
        } else {
            self.m_size = *((*stream as *mut c_long).add(*streamPos as usize / core::mem::size_of::<c_long>()));
            *streamPos += core::mem::size_of::<c_long>() as c_long;
            self.m_data = game_interface_malloc(game, self.m_size as usize);
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
            &self.m_id as *const _ as *const c_void,
            core::mem::size_of_val(&self.m_id),
            1,
            m_fileHandle,
        );
        fwrite(
            &self.m_size as *const _ as *const c_void,
            core::mem::size_of_val(&self.m_size),
            1,
            m_fileHandle,
        );
        fwrite(
            self.m_data,
            self.m_size as usize,
            1,
            m_fileHandle,
        );

        true as c_int
    }

    /*
    -------------------------
    Duplicate
    -------------------------
    */

    pub unsafe fn Duplicate(&self, icarus: *mut CIcarus) -> *mut CBlockMember {
        let mut newblock = Box::new(CBlockMember {
            m_id: -1,
            m_size: -1,
            m_data: ptr::null_mut(),
        });

        // In C++: if ( newblock == NULL ) return NULL;
        // Box::new always succeeds in Rust

        newblock.SetData_void(self.m_data, self.m_size as usize, icarus);
        newblock.SetSize(self.m_size);
        newblock.SetID(self.m_id);

        Box::into_raw(newblock)
    }
}

/*
===================================================================================================

  CBlock

===================================================================================================
*/

#[repr(C)]
pub struct CBlock {
    m_flags: c_uchar,
    m_id: c_int,
    m_members: blockMember_v,
}

impl CBlock {
    pub fn new() -> Box<CBlock> {
        Box::new(CBlock {
            m_flags: 0,
            m_id: 0,
            m_members: Vec::new(),
        })
    }

    /*
    -------------------------
    Init
    -------------------------
    */

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
        let game = icarus_get_game(icarus);
        let mut numMembers = self.GetNumMembers() as c_int;
        let mut bMember: *mut CBlockMember;

        while numMembers > 0 {
            numMembers -= 1;
            bMember = self.GetMember(numMembers);

            if bMember.is_null() {
                return false as c_int;
            }

            (*bMember).Free(game);
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
        let mut bMember = CBlockMember::new();

        bMember.SetID(member_id);

        bMember.SetData_char(member_data, icarus);
        bMember.SetSize((strlen(member_data) + 1) as c_int);

        self.AddMember(Box::into_raw(bMember));

        true as c_int
    }

    pub unsafe fn Write_vec3(&mut self, member_id: c_int, member_data: *const vec3_t, icarus: *mut CIcarus) -> c_int {
        let mut bMember = CBlockMember::new();

        bMember.SetID(member_id);
        bMember.SetData_vec3(member_data, icarus);
        bMember.SetSize(core::mem::size_of::<vec3_t>() as c_int);

        self.AddMember(Box::into_raw(bMember));

        true as c_int
    }

    pub unsafe fn Write_float(&mut self, member_id: c_int, member_data: f32, icarus: *mut CIcarus) -> c_int {
        let mut bMember = CBlockMember::new();

        bMember.SetID(member_id);
        bMember.WriteData_float(member_data, icarus);
        bMember.SetSize(core::mem::size_of_val(&member_data) as c_int);

        self.AddMember(Box::into_raw(bMember));

        true as c_int
    }

    pub unsafe fn Write_int(&mut self, member_id: c_int, member_data: c_int, icarus: *mut CIcarus) -> c_int {
        let mut bMember = CBlockMember::new();

        bMember.SetID(member_id);
        bMember.WriteData(member_data, icarus);
        bMember.SetSize(core::mem::size_of_val(&member_data) as c_int);

        self.AddMember(Box::into_raw(bMember));

        true as c_int
    }

    pub fn Write_member(&mut self, bMember: *mut CBlockMember, _icarus: *mut CIcarus) -> c_int {
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
        if memberNum > (self.GetNumMembers() as c_int) - 1 {
            return ptr::null_mut();
        }
        self.m_members[memberNum as usize]
    }

    pub fn GetNumMembers(&self) -> usize {
        self.m_members.len()
    }

    pub fn GetBlockID(&self) -> c_int {
        self.m_id
    }

    pub fn GetFlags(&self) -> c_uchar {
        self.m_flags
    }

    pub fn SetFlags(&mut self, flags: c_uchar) {
        self.m_flags = flags;
    }

    /*
    -------------------------
    GetMemberData
    -------------------------
    */

    pub unsafe fn GetMemberData(&self, memberNum: c_int) -> *mut c_void {
        if memberNum > (self.GetNumMembers() as c_int) - 1 {
            return ptr::null_mut();
        }
        (*self.GetMember(memberNum)).GetData()
    }

    /*
    -------------------------
    Duplicate
    -------------------------
    */

    pub unsafe fn Duplicate(&self, icarus: *mut CIcarus) -> *mut CBlock {
        let mut newblock = CBlock::new();

        // In C++: if ( newblock == NULL ) return false;
        // Box::new always succeeds in Rust

        newblock.Create(self.m_id);

        //Duplicate entire block and return the cc
        for mi in self.m_members.iter() {
            newblock.AddMember((*mi).Duplicate(icarus));
        }

        Box::into_raw(newblock)
    }
}

/*
===================================================================================================

  CBlockStream

===================================================================================================
*/

#[repr(C)]
pub struct CBlockStream {
    m_fileHandle: *mut c_void,
    m_fileName: [c_char; 256], // Assuming a reasonable filename buffer size
    m_stream: *mut c_char,
    m_streamPos: c_long,
    m_fileSize: c_long,
}

impl CBlockStream {
    pub const s_IBI_EXT: &'static [u8] = b".IBI"; //(I)nterpreted (B)lock (I)nstructions
    pub const s_IBI_HEADER_ID: &'static [u8] = b"IBI";
    pub const s_IBI_VERSION: f32 = 1.57;

    pub fn new() -> Box<CBlockStream> {
        Box::new(CBlockStream {
            m_fileHandle: ptr::null_mut(),
            m_fileName: [0; 256],
            m_stream: ptr::null_mut(),
            m_streamPos: 0,
            m_fileSize: 0,
        })
    }

    /*
    -------------------------
    Free
    -------------------------
    */

    pub fn Free(&mut self) -> c_int {
        //NOTENOTE: It is assumed that the user will free the passed memory block (m_stream) immediately after the run call
        //			That's why this doesn't free the memory, it only clears its internal pointer

        self.m_stream = ptr::null_mut();
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
        let mut extensionloc = strlen(filename) as c_int;
        while extensionloc >= 0 && *filename.add(extensionloc as usize) as u8 as char != '.' {
            extensionloc -= 1;
        }
        if extensionloc < 0 {
            strcpy(self.m_fileName.as_mut_ptr(), filename);
        } else {
            strncpy(
                self.m_fileName.as_mut_ptr(),
                filename,
                extensionloc as usize,
            );
            *self.m_fileName.as_mut_ptr().add(extensionloc as usize) = 0;
        }
        // add extension
        strcat(
            self.m_fileName.as_mut_ptr(),
            Self::s_IBI_EXT.as_ptr() as *const c_char,
        );

        self.m_fileHandle = fopen(
            self.m_fileName.as_ptr(),
            b"wb\0".as_ptr() as *const c_char,
        );
        if self.m_fileHandle.is_null() {
            return false as c_int;
        }

        fwrite(
            Self::s_IBI_HEADER_ID.as_ptr() as *const c_void,
            1,
            core::mem::size_of_val(Self::s_IBI_HEADER_ID),
            self.m_fileHandle,
        );
        fwrite(
            &Self::s_IBI_VERSION as *const _ as *const c_void,
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

    pub fn Init(&mut self) -> c_int {
        self.m_fileHandle = ptr::null_mut();
        memset(
            self.m_fileName.as_mut_ptr() as *mut c_void,
            0,
            core::mem::size_of_val(&self.m_fileName),
        );

        self.m_stream = ptr::null_mut();
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
        let id = (*block).GetBlockID();
        let numMembers = (*block).GetNumMembers() as c_int;
        let flags = (*block).GetFlags();

        fwrite(&id as *const _ as *const c_void, core::mem::size_of::<c_int>(), 1, self.m_fileHandle);
        fwrite(
            &numMembers as *const _ as *const c_void,
            core::mem::size_of::<c_int>(),
            1,
            self.m_fileHandle,
        );
        fwrite(
            &flags as *const _ as *const c_void,
            core::mem::size_of::<c_uchar>(),
            1,
            self.m_fileHandle,
        );

        for i in 0..numMembers {
            bMember = (*block).GetMember(i);
            (*bMember).WriteMember(self.m_fileHandle);
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
        let mut b_id: c_int;
        let mut numMembers: c_int;
        let mut flags: c_uchar;

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
                fn Z_SetNewDeleteTemporary(bTemp: c_int);
            }
            Z_SetNewDeleteTemporary(1);
        }

        while numMembers > 0 {
            numMembers -= 1;
            bMember = Box::into_raw(CBlockMember::new());
            (*bMember).ReadMember(&mut self.m_stream, &mut self.m_streamPos, icarus);
            (*get).AddMember(bMember);
        }

        #[cfg(feature = "xbox")]
        {
            extern "C" {
                fn Z_SetNewDeleteTemporary(bTemp: c_int);
            }
            Z_SetNewDeleteTemporary(0);
        }

        true as c_int
    }

    /*
    -------------------------
    Open
    -------------------------
    */

    pub unsafe fn Open(&mut self, buffer: *mut c_char, size: c_long) -> c_int {
        let mut id_header: [c_char; 4] = [0; 4]; // sizeof(s_IBI_HEADER_ID)
        let mut version: f32;

        self.Init();

        self.m_fileSize = size;

        self.m_stream = buffer;

        for i in 0..core::mem::size_of_val(&id_header) {
            id_header[i] = *(self.m_stream.add(self.m_streamPos as usize).add(i));
        }
        self.m_streamPos += core::mem::size_of_val(&id_header) as c_long;

        version = *(self.m_stream.add(self.m_streamPos as usize) as *mut f32);
        self.m_streamPos += core::mem::size_of::<f32>() as c_long;

        //Check for valid header
        if strcmp(
            id_header.as_ptr(),
            Self::s_IBI_HEADER_ID.as_ptr() as *const c_char,
        ) != 0
        {
            self.Free();
            return false as c_int;
        }

        //Check for valid version
        if version != Self::s_IBI_VERSION {
            self.Free();
            return false as c_int;
        }

        true as c_int
    }
}

// Stub constants for CIcarus
pub const CIcarus_ID_RANDOM: c_int = -1; // Adjust this based on actual value from header
