// Script Command Sequences
//
//	-- jweier

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uchar, c_void};
use std::ptr;

// Forward declarations for opaque types
#[repr(C)]
pub struct CBlock {
    _opaque: c_void,
}

#[repr(C)]
pub struct CIcarus {
    _opaque: c_void,
}

// Templates
pub type sequence_l = Vec<*mut CSequence>;
pub type block_l = Vec<*mut CBlock>;
pub type vec3_t = [f32; 3];

// External C functions
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
}

// Local stub for IIcarusInterface::GetIcarus() - returns a pointer to CIcarus
extern "C" {
    fn IIcarusInterface_GetIcarus() -> *mut CIcarus;
}

// CSequence class
#[repr(C)]
pub struct CSequence {
    // Organization information
    m_children: sequence_l,
    // sequenceID_m m_childrenMap;

    // int m_numChildren;
    m_parent: *mut CSequence,
    m_return: *mut CSequence,

    // Data information
    m_commands: block_l,
    m_flags: c_int,
    m_iterations: c_int,
    m_id: c_int,
    m_numCommands: c_int,
}

impl CSequence {
    /*
    -------------------------
    Constructors / Destructors
    -------------------------
    */

    // inline CSequence::CSequence( void )
    pub fn new() -> Box<CSequence> {
        Box::new(CSequence {
            m_numCommands: 0,
            // m_numChildren: 0,
            m_flags: 0,
            m_iterations: 1,

            m_parent: ptr::null_mut(),
            m_return: ptr::null_mut(),

            m_children: Vec::new(),
            m_commands: Vec::new(),
            m_id: 0,
        })
    }

    // CSequence::~CSequence( void )
    pub fn destroy(&mut self) {
        assert!(self.m_commands.is_empty());
        // assert!(!m_numChildren);
    }

    /*
    -------------------------
    Create
    -------------------------
    */

    // CSequence *CSequence::Create( void )
    pub fn Create() -> *mut CSequence {
        let seq = Box::new(CSequence::new());

        // TODO: Emit warning
        let seq_ptr = Box::into_raw(seq);
        if seq_ptr.is_null() {
            return ptr::null_mut();
        }

        unsafe {
            (*seq_ptr).SetFlag(SQ_COMMON);
        }

        seq_ptr
    }

    /*
    -------------------------
    Delete
    -------------------------
    */

    // void CSequence::Delete( CIcarus* icarus )
    pub unsafe fn Delete(&mut self, icarus: *mut CIcarus) {
        // Notify the parent of the deletion
        if !self.m_parent.is_null() {
            (*self.m_parent).RemoveChild(self as *mut CSequence);
        }

        // Clear all children
        if self.m_children.len() > 0 {
            for child_ptr in self.m_children.iter() {
                (*(*child_ptr)).SetParent(ptr::null_mut());
            }
        }
        self.m_children.clear();

        // Clear all held commands
        for cmd_ptr in self.m_commands.iter() {
            (*(*cmd_ptr)).Free(icarus);
            let _ = Box::from_raw(*cmd_ptr); // Free() handled internally -- not any more!!
        }

        self.m_commands.clear();
    }

    /*
    -------------------------
    AddChild
    -------------------------
    */

    // void CSequence::AddChild( CSequence *child )
    pub fn AddChild(&mut self, child: *mut CSequence) {
        assert!(!child.is_null());
        if child.is_null() {
            return;
        }

        self.m_children.insert(self.m_children.len(), child);
    }

    /*
    -------------------------
    RemoveChild
    -------------------------
    */

    // void CSequence::RemoveChild( CSequence *child )
    pub fn RemoveChild(&mut self, child: *mut CSequence) {
        assert!(!child.is_null());
        if child.is_null() {
            return;
        }

        // Remove the child from the vector
        self.m_children.retain(|&c| c != child);
    }

    /*
    -------------------------
    HasChild
    -------------------------
    */

    // bool CSequence::HasChild( CSequence *sequence )
    pub fn HasChild(&self, sequence: *mut CSequence) -> bool {
        for ci in self.m_children.iter() {
            unsafe {
                if (*ci) == sequence {
                    return true;
                }

                if (*(*ci)).HasChild(sequence) {
                    return true;
                }
            }
        }

        false
    }

    /*
    -------------------------
    SetParent
    -------------------------
    */

    // void CSequence::SetParent( CSequence *parent )
    pub fn SetParent(&mut self, parent: *mut CSequence) {
        self.m_parent = parent;

        if parent.is_null() {
            return;
        }

        unsafe {
            // Inherit the parent's properties (this avoids messy tree walks later on)
            if (*parent).m_flags & SQ_RETAIN != 0 {
                self.m_flags |= SQ_RETAIN;
            }

            if (*parent).m_flags & SQ_PENDING != 0 {
                self.m_flags |= SQ_PENDING;
            }
        }
    }

    /*
    -------------------------
    PopCommand
    -------------------------
    */

    // CBlock *CSequence::PopCommand( int type )
    pub fn PopCommand(&mut self, type_: c_int) -> *mut CBlock {
        // Make sure everything is ok
        assert!(type_ == POP_FRONT || type_ == POP_BACK);

        if self.m_commands.is_empty() {
            return ptr::null_mut();
        }

        match type_ {
            POP_FRONT => {
                let command = self.m_commands.remove(0);
                self.m_numCommands -= 1;
                command
            }

            POP_BACK => {
                let command = self.m_commands.pop().unwrap();
                self.m_numCommands -= 1;
                command
            }

            _ => {
                // Invalid flag
                ptr::null_mut()
            }
        }
    }

    /*
    -------------------------
    PushCommand
    -------------------------
    */

    // int CSequence::PushCommand( CBlock *block, int type )
    pub fn PushCommand(&mut self, block: *mut CBlock, type_: c_int) -> c_int {
        // Make sure everything is ok
        assert!(type_ == PUSH_FRONT || type_ == PUSH_BACK);
        assert!(!block.is_null());

        match type_ {
            PUSH_FRONT => {
                self.m_commands.insert(0, block);
                self.m_numCommands += 1;
                1
            }

            PUSH_BACK => {
                self.m_commands.insert(self.m_commands.len(), block);
                self.m_numCommands += 1;
                1
            }

            _ => {
                // Invalid flag
                0
            }
        }
    }

    /*
    -------------------------
    SetFlag
    -------------------------
    */

    // void CSequence::SetFlag( int flag )
    pub fn SetFlag(&mut self, flag: c_int) {
        self.m_flags |= flag;
    }

    /*
    -------------------------
    RemoveFlag
    -------------------------
    */

    // void CSequence::RemoveFlag( int flag, bool children )
    pub fn RemoveFlag(&mut self, flag: c_int, children: bool) {
        self.m_flags &= !flag;

        if children {
            for si in self.m_children.iter() {
                unsafe {
                    (*(*si)).RemoveFlag(flag, true);
                }
            }
        }
    }

    /*
    -------------------------
    HasFlag
    -------------------------
    */

    // int CSequence::HasFlag( int flag )
    pub fn HasFlag(&self, flag: c_int) -> c_int {
        (self.m_flags & flag)
    }

    /*
    -------------------------
    SetReturn
    -------------------------
    */

    // void CSequence::SetReturn ( CSequence *sequence )
    pub fn SetReturn(&mut self, sequence: *mut CSequence) {
        assert!(sequence != (self as *mut CSequence));
        self.m_return = sequence;
    }

    /*
    -------------------------
    GetChildByID
    -------------------------
    */

    // CSequence *CSequence::GetChildByID( int id )
    pub fn GetChildByID(&self, id: c_int) -> *mut CSequence {
        if id < 0 {
            return ptr::null_mut();
        }

        // NOTENOTE: Done for safety reasons, I don't know what this template will return on underflow ( sigh... )
        for iterSeq in self.m_children.iter() {
            unsafe {
                if (*(*iterSeq)).GetID() == id {
                    return *iterSeq;
                }
            }
        }

        ptr::null_mut()
    }

    /*
    -------------------------
    GetChildByIndex
    -------------------------
    */

    // CSequence *CSequence::GetChildByIndex( int iIndex )
    pub fn GetChildByIndex(&self, iIndex: c_int) -> *mut CSequence {
        if iIndex < 0 || iIndex >= (self.m_children.len() as c_int) {
            return ptr::null_mut();
        }

        self.m_children[iIndex as usize]
    }

    /*
    -------------------------
    SaveCommand
    -------------------------
    */

    // int CSequence::SaveCommand( CBlock *block )
    pub unsafe fn SaveCommand(&self, block: *mut CBlock) -> c_int {
        let pIcarus = IIcarusInterface_GetIcarus() as *mut CIcarus;

        // Data saved here (IBLK):
        //	Block ID.
        //	Block Flags.
        //	Number of Block Members.
        //	Block Members:
        //				- Block Member ID.
        //				- Block Data Size.
        //				- Block (Raw) Data.

        // Save out the block ID
        let bID = (*block).GetBlockID();
        (*pIcarus).BufferWrite(&bID as *const _ as *mut c_void, core::mem::size_of_val(&bID));

        // Save out the block's flags
        let flags = (*block).GetFlags();
        (*pIcarus).BufferWrite(&flags as *const _ as *mut c_void, core::mem::size_of_val(&flags));

        // Save out the number of members to read
        let numMembers = (*block).GetNumMembers() as c_int;
        (*pIcarus).BufferWrite(&numMembers as *const _ as *mut c_void, core::mem::size_of_val(&numMembers));

        for i in 0..numMembers {
            let bm = (*block).GetMember(i);

            // Save the block id
            let bID = (*bm).GetID();
            (*pIcarus).BufferWrite(&bID as *const _ as *mut c_void, core::mem::size_of_val(&bID));

            // Save out the data size
            let size = (*bm).GetSize();
            (*pIcarus).BufferWrite(&size as *const _ as *mut c_void, core::mem::size_of_val(&size));

            // Save out the raw data
            (*pIcarus).BufferWrite((*bm).GetData(), size as usize);
        }

        1
    }

    // int CSequence::LoadCommand( CBlock *block, CIcarus *icarus )
    pub unsafe fn LoadCommand(&self, block: *mut CBlock, icarus: *mut CIcarus) -> c_int {
        let game = (*icarus).GetGame();
        let mut bID: c_int = 0;
        let mut bSize: c_int = 0;
        let mut bData: *mut c_void = ptr::null_mut();
        let mut flags: c_uchar = 0;
        let mut id: c_int = 0;
        let mut numMembers: c_int = 0;

        // Data expected/loaded here (IBLK) (with the size as : 'IBSZ' ).
        //	Block ID.
        //	Block Flags.
        //	Number of Block Members.
        //	Block Members:
        //				- Block Member ID.
        //				- Block Data Size.
        //				- Block (Raw) Data.

        // Get the block ID.
        (*icarus).BufferRead(&mut id as *mut _ as *mut c_void, core::mem::size_of_val(&id));
        (*block).Create(id);

        // Read the block's flags
        (*icarus).BufferRead(&mut flags as *mut _ as *mut c_void, core::mem::size_of_val(&flags));
        (*block).SetFlags(flags);

        // Get the number of block members
        (*icarus).BufferRead(&mut numMembers as *mut _ as *mut c_void, core::mem::size_of_val(&numMembers));

        for j in 0..numMembers {
            // Get the member ID
            (*icarus).BufferRead(&mut bID as *mut _ as *mut c_void, core::mem::size_of_val(&bID));

            // Get the member size
            (*icarus).BufferRead(&mut bSize as *mut _ as *mut c_void, core::mem::size_of_val(&bSize));

            // Get the member's data
            bData = (*game).Malloc(bSize as usize);
            if bData.is_null() {
                return 0;
            }

            // Get the actual raw data
            (*icarus).BufferRead(bData, bSize as usize);

            // Write out the correct type
            match bID {
                TK_INT => {
                    assert!(false);
                    let data = *(bData as *const c_int);
                    (*block).Write_float(TK_FLOAT, data as f32, icarus);
                }

                TK_FLOAT => {
                    (*block).Write_float(TK_FLOAT, *(bData as *const f32), icarus);
                }

                TK_STRING | TK_IDENTIFIER | TK_CHAR => {
                    (*block).Write_char(TK_STRING, bData as *const c_char, icarus);
                }

                TK_VECTOR | TK_VECTOR_START => {
                    (*block).Write_vec3(TK_VECTOR, bData as *const vec3_t, icarus);
                }

                ID_TAG => {
                    (*block).Write_float(ID_TAG, ID_TAG as f32, icarus);
                }

                ID_GET => {
                    (*block).Write_float(ID_GET, ID_GET as f32, icarus);
                }

                ID_RANDOM => {
                    (*block).Write_float(ID_RANDOM, *(bData as *const f32), icarus); // (float) ID_RANDOM );
                }

                TK_EQUALS | TK_GREATER_THAN | TK_LESS_THAN | TK_NOT => {
                    (*block).Write_float(bID, 0.0, icarus);
                }

                _ => {
                    assert!(false);
                    (*game).Free(bData);
                    return 0;
                }
            }

            // Get rid of the temp memory
            (*game).Free(bData);
        }

        1
    }

    /*
    -------------------------
    Save
    -------------------------
    */

    // int CSequence::Save()
    pub unsafe fn Save(&self) -> c_int {
        // Data saved here.
        //	Parent ID.
        //	Return ID.
        //	Number of Children.
        //	Children.
        //			- Child ID
        //	Save Flags.
        //	Save Iterations.
        //	Number of Commands
        //			- Commands (raw) data.

        let pIcarus = IIcarusInterface_GetIcarus() as *mut CIcarus;

        // Save the parent (by GUID).
        let id = if !self.m_parent.is_null() {
            (*self.m_parent).GetID()
        } else {
            -1
        };
        (*pIcarus).BufferWrite(&id as *const _ as *mut c_void, core::mem::size_of_val(&id));

        // Save the return (by GUID)
        let id = if !self.m_return.is_null() {
            (*self.m_return).GetID()
        } else {
            -1
        };
        (*pIcarus).BufferWrite(&id as *const _ as *mut c_void, core::mem::size_of_val(&id));

        // Save the number of children
        let iNumChildren = self.m_children.len() as c_int;
        (*pIcarus).BufferWrite(&iNumChildren as *const _ as *mut c_void, core::mem::size_of_val(&iNumChildren));

        // Save out the children (only by GUID)
        for iterSeq in self.m_children.iter() {
            let id = (*(*iterSeq)).GetID();
            (*pIcarus).BufferWrite(&id as *const _ as *mut c_void, core::mem::size_of_val(&id));
        }

        // Save flags
        (*pIcarus).BufferWrite(&self.m_flags as *const _ as *mut c_void, core::mem::size_of_val(&self.m_flags));

        // Save iterations
        (*pIcarus).BufferWrite(&self.m_iterations as *const _ as *mut c_void, core::mem::size_of_val(&self.m_iterations));

        // Save the number of commands
        (*pIcarus).BufferWrite(&self.m_numCommands as *const _ as *mut c_void, core::mem::size_of_val(&self.m_numCommands));

        // Save the commands
        for bi in self.m_commands.iter() {
            self.SaveCommand(*bi);
        }

        1
    }

    /*
    -------------------------
    Load
    -------------------------
    */

    // int CSequence::Load( CIcarus* icarus )
    pub unsafe fn Load(&mut self, icarus: *mut CIcarus) -> c_int {
        let mut sequence: *mut CSequence = ptr::null_mut();
        let mut block: *mut CBlock = ptr::null_mut();
        let mut id: c_int = 0;

        // Data expected/loaded here (ISEQ) (with the size as : 'ISSZ' ).
        //	Parent ID.
        //	Return ID.
        //	Number of Children.
        //	Children.
        //			- Child ID
        //	Save Flags.
        //	Save Iterations.
        //	Number of Commands
        //			- Commands (raw) data.

        // Get the parent sequence
        (*icarus).BufferRead(&mut id as *mut _ as *mut c_void, core::mem::size_of_val(&id));
        self.m_parent = if id != -1 { (*icarus).GetSequence(id) } else { ptr::null_mut() };

        // Get the return sequence
        (*icarus).BufferRead(&mut id as *mut _ as *mut c_void, core::mem::size_of_val(&id));
        self.m_return = if id != -1 { (*icarus).GetSequence(id) } else { ptr::null_mut() };

        // Get the number of children
        let mut iNumChildren: c_int = 0;
        (*icarus).BufferRead(&mut iNumChildren as *mut _ as *mut c_void, core::mem::size_of_val(&iNumChildren));

        // Reload all children
        for i in 0..iNumChildren {
            // Get the child sequence ID
            (*icarus).BufferRead(&mut id as *mut _ as *mut c_void, core::mem::size_of_val(&id));

            // Get the desired sequence
            sequence = (*icarus).GetSequence(id);
            if sequence.is_null() {
                return 0;
            }

            // Insert this into the list
            self.m_children.insert(self.m_children.len(), sequence);

            // Restore the connection in the child / ID map
            // m_childrenMap[ i ] = sequence;
        }

        // Get the sequence flags
        (*icarus).BufferRead(&mut self.m_flags as *mut _ as *mut c_void, core::mem::size_of_val(&self.m_flags));

        // Get the number of iterations
        (*icarus).BufferRead(&mut self.m_iterations as *mut _ as *mut c_void, core::mem::size_of_val(&self.m_iterations));

        let mut numCommands: c_int = 0;

        // Get the number of commands
        (*icarus).BufferRead(&mut numCommands as *mut _ as *mut c_void, core::mem::size_of_val(&numCommands));

        // Get all the commands
        for i in 0..numCommands {
            block = Box::into_raw(Box::new(CBlock::new()));
            self.LoadCommand(block, icarus);

            // Save the block
            self.PushCommand(block, PUSH_BACK);
        }

        1
    }

    // Getter and Setter methods
    pub fn GetParent(&self) -> *mut CSequence {
        self.m_parent
    }

    pub fn GetReturn(&self) -> *mut CSequence {
        self.m_return
    }

    pub fn GetID(&self) -> c_int {
        self.m_id
    }

    pub fn SetID(&mut self, id: c_int) {
        self.m_id = id;
    }

    pub fn GetIterations(&self) -> c_int {
        self.m_iterations
    }

    pub fn SetIterations(&mut self, it: c_int) {
        self.m_iterations = it;
    }

    pub fn GetFlags(&self) -> c_int {
        self.m_flags
    }

    pub fn SetFlags(&mut self, flags: c_int) {
        self.m_flags = flags;
    }

    pub fn GetNumCommands(&self) -> c_int {
        self.m_numCommands
    }

    pub fn GetNumChildren(&self) -> c_int {
        self.m_children.len() as c_int
    }
}

impl Drop for CSequence {
    fn drop(&mut self) {
        self.destroy();
    }
}

// Enum constants
pub const SQ_COMMON: c_int = 0x00000000;           // Common one-pass sequence
pub const SQ_LOOP: c_int = 0x00000001;             // Looping sequence
pub const SQ_RETAIN: c_int = 0x00000002;           // Inside a looping sequence list, retain the information
pub const SQ_AFFECT: c_int = 0x00000004;           // Affect sequence
pub const SQ_RUN: c_int = 0x00000008;              // A run block
pub const SQ_PENDING: c_int = 0x00000010;          // Pending use, don't free when flushing the sequences
pub const SQ_CONDITIONAL: c_int = 0x00000020;      // Conditional statement
pub const SQ_TASK: c_int = 0x00000040;             // Task block

pub const POP_FRONT: c_int = 0;
pub const POP_BACK: c_int = 1;
pub const PUSH_FRONT: c_int = 2;
pub const PUSH_BACK: c_int = 3;

// Token IDs from CIcarus enum - TK_*
pub const TK_EOF: c_int = -1;
pub const TK_UNDEFINED: c_int = 0;
pub const TK_COMMENT: c_int = 1;
pub const TK_EOL: c_int = 2;
pub const TK_CHAR: c_int = 3;
pub const TK_STRING: c_int = 4;
pub const TK_INT: c_int = 5;
pub const TK_INTEGER: c_int = 5; // TK_INT
pub const TK_FLOAT: c_int = 6;
pub const TK_IDENTIFIER: c_int = 7;
pub const TK_USERDEF: c_int = 8;
pub const TK_BLOCK_START: c_int = 8; // TK_USERDEF
pub const TK_BLOCK_END: c_int = 9;
pub const TK_VECTOR_START: c_int = 10;
pub const TK_VECTOR_END: c_int = 11;
pub const TK_OPEN_PARENTHESIS: c_int = 12;
pub const TK_CLOSED_PARENTHESIS: c_int = 13;
pub const TK_VECTOR: c_int = 14;
pub const TK_GREATER_THAN: c_int = 15;
pub const TK_LESS_THAN: c_int = 16;
pub const TK_EQUALS: c_int = 17;
pub const TK_NOT: c_int = 18;

pub const NUM_USER_TOKENS: c_int = 19;

// ID constants from CIcarus enum - ID_*
pub const ID_AFFECT: c_int = NUM_USER_TOKENS;
pub const ID_SOUND: c_int = 20;
pub const ID_MOVE: c_int = 21;
pub const ID_ROTATE: c_int = 22;
pub const ID_WAIT: c_int = 23;
pub const ID_BLOCK_START: c_int = 24;
pub const ID_BLOCK_END: c_int = 25;
pub const ID_SET: c_int = 26;
pub const ID_LOOP: c_int = 27;
pub const ID_LOOPEND: c_int = 28;
pub const ID_PRINT: c_int = 29;
pub const ID_USE: c_int = 30;
pub const ID_FLUSH: c_int = 31;
pub const ID_RUN: c_int = 32;
pub const ID_KILL: c_int = 33;
pub const ID_REMOVE: c_int = 34;
pub const ID_CAMERA: c_int = 35;
pub const ID_GET: c_int = 36;
pub const ID_RANDOM: c_int = 37;
pub const ID_IF: c_int = 38;
pub const ID_ELSE: c_int = 39;
pub const ID_REM: c_int = 40;
pub const ID_TASK: c_int = 41;
pub const ID_DO: c_int = 42;
pub const ID_DECLARE: c_int = 43;
pub const ID_FREE: c_int = 44;
pub const ID_DOWAIT: c_int = 45;
pub const ID_SIGNAL: c_int = 46;
pub const ID_WAITSIGNAL: c_int = 47;
pub const ID_PLAY: c_int = 48;
pub const ID_TAG: c_int = 49;
pub const ID_EOF: c_int = 50;
pub const NUM_IDS: c_int = 51;

// Stubs for CBlock and CIcarus methods needed by Sequence.cpp
// These are declarations that will be linked externally

impl CBlock {
    pub fn GetBlockID(&self) -> c_int {
        0 // stub
    }

    pub fn GetFlags(&self) -> c_uchar {
        0 // stub
    }

    pub fn SetFlags(&mut self, _flags: c_uchar) {
        // stub
    }

    pub fn GetNumMembers(&self) -> usize {
        0 // stub
    }

    pub fn GetMember(&self, _memberNum: c_int) -> *mut CBlockMember {
        ptr::null_mut() // stub
    }

    pub fn Create(&mut self, _id: c_int) {
        // stub
    }

    pub unsafe fn Free(&mut self, _icarus: *mut CIcarus) -> c_int {
        // stub
        1
    }

    pub unsafe fn Write_char(&mut self, _member_id: c_int, _member_data: *const c_char, _icarus: *mut CIcarus) -> c_int {
        // stub
        1
    }

    pub unsafe fn Write_vec3(&mut self, _member_id: c_int, _member_data: *const vec3_t, _icarus: *mut CIcarus) -> c_int {
        // stub
        1
    }

    pub unsafe fn Write_float(&mut self, _member_id: c_int, _member_data: f32, _icarus: *mut CIcarus) -> c_int {
        // stub
        1
    }

    pub unsafe fn Write_int(&mut self, _member_id: c_int, _member_data: c_int, _icarus: *mut CIcarus) -> c_int {
        // stub
        1
    }

    pub fn new() -> Self {
        CBlock {
            _opaque: unsafe { core::mem::zeroed() },
        }
    }
}

#[repr(C)]
pub struct CBlockMember {
    pub m_id: c_int,         // ID of the value contained in data
    pub m_size: c_int,       // Size of the data member variable
    pub m_data: *mut c_void, // Data for this member
}

impl CBlockMember {
    pub fn new() -> Box<CBlockMember> {
        Box::new(CBlockMember {
            m_id: -1,
            m_size: -1,
            m_data: ptr::null_mut(),
        })
    }

    pub fn GetID(&self) -> c_int {
        self.m_id
    }

    pub fn GetSize(&self) -> c_int {
        self.m_size
    }

    pub fn GetData(&self) -> *mut c_void {
        self.m_data
    }
}

impl CIcarus {
    pub unsafe fn BufferWrite(&mut self, _data: *mut c_void, _size: usize) {
        // stub
    }

    pub unsafe fn BufferRead(&mut self, _data: *mut c_void, _size: usize) {
        // stub
    }

    pub fn GetGame(&self) -> *mut IGameInterface {
        ptr::null_mut() // stub
    }

    pub fn GetSequence(&self, _id: c_int) -> *mut CSequence {
        ptr::null_mut() // stub
    }
}

#[repr(C)]
pub struct IGameInterface {
    _opaque: c_void,
}

impl IGameInterface {
    pub unsafe fn Malloc(&mut self, _size: usize) -> *mut c_void {
        ptr::null_mut() // stub
    }

    pub unsafe fn Free(&mut self, _ptr: *mut c_void) {
        // stub
    }
}
