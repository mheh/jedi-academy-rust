// IcarusImplementation.h
#![allow(non_snake_case)]

// pragma warning( disable : 4786 )	// identifier was truncated
// pragma warning (push, 3)			// go back down to 3 for the stl include
// pragma warning (disable:4503)		// decorated name length xceeded, name was truncated
// #include <string>
// #include <vector>
// #include <map>
// #include <list>
// #include <algorithm>
// pragma warning (pop)
// pragma warning (disable:4503)		// decorated name length xceeded, name was truncated
// using namespace std;

use core::ffi::c_int;
use std::collections::{HashMap, VecDeque};
use std::ffi::CStr;

// Forward declarations
pub struct CSequence;
pub struct CSequencer;
pub struct CIcarusSequencer;
pub struct CIcarusSequence;

// LOCAL STUB: IIcarusInterface and IGameInterface
pub struct IIcarusInterface;
pub struct IGameInterface;

impl IGameInterface {
    pub fn GetGame(flavor: c_int) -> *mut IGameInterface {
        std::ptr::null_mut()
    }

    pub fn Malloc(&mut self, size: usize) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }

    pub fn Free(&mut self, ptr: *mut std::ffi::c_void) {}
}

#[repr(C)]
pub struct CIcarus {
    pub m_flavor: c_int,
    pub m_nextSequencerID: c_int,
    pub m_GUID: c_int,

    pub m_sequences: Vec<*mut CSequence>,
    pub m_sequencers: Vec<*mut CSequencer>,
    pub m_sequencerMap: HashMap<c_int, *mut CSequencer>,

    pub m_signals: HashMap<String, u8>,

    // DEBUG members
    #[cfg(debug_assertions)]
    pub m_DEBUG_NumSequencerAlloc: c_int,
    #[cfg(debug_assertions)]
    pub m_DEBUG_NumSequencerFreed: c_int,
    #[cfg(debug_assertions)]
    pub m_DEBUG_NumSequencerResidual: c_int,

    #[cfg(debug_assertions)]
    pub m_DEBUG_NumSequenceAlloc: c_int,
    #[cfg(debug_assertions)]
    pub m_DEBUG_NumSequenceFreed: c_int,
    #[cfg(debug_assertions)]
    pub m_DEBUG_NumSequenceResidual: c_int,

    // Used by the new Icarus Save code.
    pub m_ulBufferCurPos: u64,
    pub m_ulBytesRead: u64,
    pub m_byBuffer: *mut u8,
}

impl CIcarus {
    pub const MAX_STRING_SIZE: c_int = 256;
    pub const MAX_FILENAME_LENGTH: c_int = 1024;

    pub const MAX_BUFFER_SIZE: u64 = 100000;

    pub fn new(flavor: c_int) -> *mut CIcarus {
        std::ptr::null_mut()
    }

    // inline IGameInterface* GetGame() {return IGameInterface::GetGame(m_flavor);};
    pub fn GetGame(&mut self) -> *mut IGameInterface {
        unsafe { IGameInterface::GetGame(self.m_flavor) }
    }

    // mandatory overrides
    // Get the current Game flavor.
    pub fn GetFlavor(&self) -> c_int {
        self.m_flavor
    }

    pub fn Save(&mut self) -> c_int {
        0
    }

    pub fn Load(&mut self) -> c_int {
        0
    }

    pub fn Run(&mut self, icarusID: c_int, buffer: *mut i8, length: i64) -> c_int {
        0
    }

    pub fn DeleteIcarusID(&mut self, icarusID: &mut c_int) {
        *icarusID = 0;
    }

    pub fn GetIcarusID(&self, ownerID: c_int) -> c_int {
        0
    }

    pub fn Update(&mut self, icarusID: c_int) -> c_int {
        0
    }

    pub fn IsRunning(&self, icarusID: c_int) -> c_int {
        0
    }

    pub fn Completed(&mut self, icarusID: c_int, taskID: c_int) {}

    pub fn Precache(&mut self, buffer: *mut i8, length: i64) {}

    pub fn Delete(&mut self) {}

    pub fn Free(&mut self) {}

    pub fn GetSequence(&mut self, id: c_int) -> *mut CSequence {
        std::ptr::null_mut()
    }

    pub fn GetSequence_noarg(&mut self) -> *mut CSequence {
        std::ptr::null_mut()
    }

    pub fn DeleteSequence(&mut self, sequence: *mut CSequence) {}

    pub fn AllocateSequences(&mut self, numSequences: c_int, idTable: *mut c_int) -> c_int {
        0
    }

    pub fn FindSequencer(&mut self, sequencerID: c_int) -> *mut CSequencer {
        std::ptr::null_mut()
    }

    pub fn SaveSequenceIDTable(&mut self) -> c_int {
        0
    }

    pub fn SaveSequences(&mut self) -> c_int {
        0
    }

    pub fn SaveSequencers(&mut self) -> c_int {
        0
    }

    pub fn SaveSignals(&mut self) -> c_int {
        0
    }

    pub fn LoadSignals(&mut self) -> c_int {
        0
    }

    pub fn LoadSequence(&mut self) -> c_int {
        0
    }

    pub fn LoadSequences(&mut self) -> c_int {
        0
    }

    pub fn LoadSequencers(&mut self) -> c_int {
        0
    }

    pub fn Signal(&mut self, identifier: *const i8) {}

    pub fn CheckSignal(&self, identifier: *const i8) -> bool {
        false
    }

    pub fn ClearSignal(&mut self, identifier: *const i8) {}

    // Destroy the File Buffer.
    pub fn DestroyBuffer(&mut self) {}

    // Create the File Buffer.
    pub fn CreateBuffer(&mut self) {}

    // Reset the buffer completely.
    pub fn ResetBuffer(&mut self) {}

    // Write to a buffer.
    pub fn BufferWrite(&mut self, pSrcData: *const std::ffi::c_void, ulNumBytesToWrite: u64) {}

    // Read from a buffer.
    pub fn BufferRead(&mut self, pDstBuff: *mut std::ffi::c_void, ulNumBytesToRead: u64) {}
}

impl CIcarus {
    pub const TK_EOF: c_int = -1;
    pub const TK_UNDEFINED: c_int = 0;
    pub const TK_COMMENT: c_int = 1;
    pub const TK_EOL: c_int = 2;
    pub const TK_CHAR: c_int = 3;
    pub const TK_STRING: c_int = 4;
    pub const TK_INT: c_int = 5;
    pub const TK_INTEGER: c_int = Self::TK_INT;
    pub const TK_FLOAT: c_int = 6;
    pub const TK_IDENTIFIER: c_int = 7;
    pub const TK_USERDEF: c_int = 8;
    pub const TK_BLOCK_START: c_int = Self::TK_USERDEF;
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

    // ID defines
    pub const ID_AFFECT: c_int = Self::NUM_USER_TOKENS;
    pub const ID_SOUND: c_int = Self::NUM_USER_TOKENS + 1;
    pub const ID_MOVE: c_int = Self::NUM_USER_TOKENS + 2;
    pub const ID_ROTATE: c_int = Self::NUM_USER_TOKENS + 3;
    pub const ID_WAIT: c_int = Self::NUM_USER_TOKENS + 4;
    pub const ID_BLOCK_START: c_int = Self::NUM_USER_TOKENS + 5;
    pub const ID_BLOCK_END: c_int = Self::NUM_USER_TOKENS + 6;
    pub const ID_SET: c_int = Self::NUM_USER_TOKENS + 7;
    pub const ID_LOOP: c_int = Self::NUM_USER_TOKENS + 8;
    pub const ID_LOOPEND: c_int = Self::NUM_USER_TOKENS + 9;
    pub const ID_PRINT: c_int = Self::NUM_USER_TOKENS + 10;
    pub const ID_USE: c_int = Self::NUM_USER_TOKENS + 11;
    pub const ID_FLUSH: c_int = Self::NUM_USER_TOKENS + 12;
    pub const ID_RUN: c_int = Self::NUM_USER_TOKENS + 13;
    pub const ID_KILL: c_int = Self::NUM_USER_TOKENS + 14;
    pub const ID_REMOVE: c_int = Self::NUM_USER_TOKENS + 15;
    pub const ID_CAMERA: c_int = Self::NUM_USER_TOKENS + 16;
    pub const ID_GET: c_int = Self::NUM_USER_TOKENS + 17;
    pub const ID_RANDOM: c_int = Self::NUM_USER_TOKENS + 18;
    pub const ID_IF: c_int = Self::NUM_USER_TOKENS + 19;
    pub const ID_ELSE: c_int = Self::NUM_USER_TOKENS + 20;
    pub const ID_REM: c_int = Self::NUM_USER_TOKENS + 21;
    pub const ID_TASK: c_int = Self::NUM_USER_TOKENS + 22;
    pub const ID_DO: c_int = Self::NUM_USER_TOKENS + 23;
    pub const ID_DECLARE: c_int = Self::NUM_USER_TOKENS + 24;
    pub const ID_FREE: c_int = Self::NUM_USER_TOKENS + 25;
    pub const ID_DOWAIT: c_int = Self::NUM_USER_TOKENS + 26;
    pub const ID_SIGNAL: c_int = Self::NUM_USER_TOKENS + 27;
    pub const ID_WAITSIGNAL: c_int = Self::NUM_USER_TOKENS + 28;
    pub const ID_PLAY: c_int = Self::NUM_USER_TOKENS + 29;
    pub const ID_TAG: c_int = Self::NUM_USER_TOKENS + 30;
    pub const ID_EOF: c_int = Self::NUM_USER_TOKENS + 31;
    pub const NUM_IDS: c_int = Self::NUM_USER_TOKENS + 32;

    // Type defines
    pub const TYPE_WAIT_COMPLETE: c_int = Self::NUM_IDS;
    pub const TYPE_WAIT_TRIGGERED: c_int = Self::NUM_IDS + 1;

    pub const TYPE_ANGLES: c_int = Self::NUM_IDS + 2;
    pub const TYPE_ORIGIN: c_int = Self::NUM_IDS + 3;

    pub const TYPE_INSERT: c_int = Self::NUM_IDS + 4;
    pub const TYPE_FLUSH: c_int = Self::NUM_IDS + 5;

    pub const TYPE_PAN: c_int = Self::NUM_IDS + 6;
    pub const TYPE_ZOOM: c_int = Self::NUM_IDS + 7;
    pub const TYPE_MOVE: c_int = Self::NUM_IDS + 8;
    pub const TYPE_FADE: c_int = Self::NUM_IDS + 9;
    pub const TYPE_PATH: c_int = Self::NUM_IDS + 10;
    pub const TYPE_ENABLE: c_int = Self::NUM_IDS + 11;
    pub const TYPE_DISABLE: c_int = Self::NUM_IDS + 12;
    pub const TYPE_SHAKE: c_int = Self::NUM_IDS + 13;
    pub const TYPE_ROLL: c_int = Self::NUM_IDS + 14;
    pub const TYPE_TRACK: c_int = Self::NUM_IDS + 15;
    pub const TYPE_DISTANCE: c_int = Self::NUM_IDS + 16;
    pub const TYPE_FOLLOW: c_int = Self::NUM_IDS + 17;

    pub const TYPE_VARIABLE: c_int = Self::NUM_IDS + 18;

    pub const TYPE_EOF: c_int = Self::NUM_IDS + 19;
    pub const NUM_TYPES: c_int = Self::NUM_IDS + 20;
}

pub static mut ICARUS_VERSION: f64 = 0.0;

pub static mut s_flavorsAvailable: c_int = 0;
pub static mut s_instances: *mut *mut CIcarus = std::ptr::null_mut();
