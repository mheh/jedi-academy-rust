// Sequencer Header File

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_long, c_void};

// Forward declarations
pub struct ICARUS_Instance;
pub struct CSequence;
pub struct CTaskGroup;
pub struct CTaskManager;
pub struct CBlock;
pub struct CBlockStream;
pub struct interface_export_t;

// Defines

pub const SQ_COMMON: c_int = 0x00000000;        // Common one-pass sequence
pub const SQ_LOOP: c_int = 0x00000001;          // Looping sequence
pub const SQ_RETAIN: c_int = 0x00000002;        // Inside a looping sequence list, retain the information
pub const SQ_AFFECT: c_int = 0x00000004;        // Affect sequence
pub const SQ_RUN: c_int = 0x00000008;           // A run block
pub const SQ_PENDING: c_int = 0x00000010;       // Pending use, don't free when flushing the sequences
pub const SQ_CONDITIONAL: c_int = 0x00000020;   // Conditional statement
pub const SQ_TASK: c_int = 0x00000040;          // Task block

pub const BF_ELSE: c_int = 0x00000001;          // Block has an else id	//FIXME: This was a sloppy fix for a problem that arose from conditionals

// Macro
#[inline]
pub fn S_FAILED(a: c_int) -> bool {
    a != SEQ_OK
}

// Typedefs

#[repr(C)]
pub struct bstream_s {
    pub stream: *mut CBlockStream,
    pub last: *mut bstream_s,
}

pub type bstream_t = bstream_s;

// Enumerations

pub const SEQ_OK: c_int = 0;      // Command was successfully added
pub const SEQ_FAILED: c_int = 1;  // An error occured while trying to insert the command

// Type aliases for C++ STL containers
// The actual layout is implementation-dependent and must be defined in C++
pub type sequence_l = c_void;           // std::list<CSequence*>
pub type taskSequence_m = c_void;       // std::map<CTaskGroup*, CSequence*>

// Sequencer

/*
==================================================================================================

  CSequencer

==================================================================================================
*/

#[repr(C)]
pub struct CSequencer {
    // typedef	map < int, CSequence * >			sequenceID_m;
    // typedef list < CSequence * >				sequence_l;
    // typedef map < CTaskGroup *, CSequence * >	taskSequence_m;

    // Member variables

    m_owner: *mut ICARUS_Instance,
    m_ownerID: c_int,

    m_taskManager: *mut CTaskManager,
    m_ie: *mut interface_export_t,                  // This is unique to the sequencer so that client side and server side sequencers could both
                                                     // operate under different interfaces (for client side scripting)

    m_numCommands: c_int,                           // Total number of commands for the sequencer (including all child sequences)

    // sequenceID_m		m_sequenceMap;
    m_sequences: *mut sequence_l,
    m_taskSequences: *mut taskSequence_m,

    m_curSequence: *mut CSequence,
    m_curGroup: *mut CTaskGroup,

    m_curStream: *mut bstream_t,

    m_elseValid: c_int,
    m_elseOwner: *mut CBlock,
    m_streamsCreated: *mut *mut bstream_t,
}

impl CSequencer {
    pub fn new() -> *mut CSequencer {
        unimplemented!()
    }

    pub fn Create() -> *mut CSequencer {
        unimplemented!()
    }

    pub fn Init(
        &mut self,
        ownerID: c_int,
        ie: *mut interface_export_t,
        taskManager: *mut CTaskManager,
        iCARUS: *mut ICARUS_Instance,
    ) -> c_int {
        unimplemented!()
    }

    pub fn Free(&mut self) -> c_int {
        unimplemented!()
    }

    pub fn Run(&mut self, buffer: *mut c_char, size: c_long) -> c_int {
        unimplemented!()
    }

    pub fn Callback(
        &mut self,
        taskManager: *mut CTaskManager,
        block: *mut CBlock,
        returnCode: c_int,
    ) -> c_int {
        unimplemented!()
    }

    #[inline]
    pub fn GetOwner(&self) -> *mut ICARUS_Instance {
        self.m_owner
    }

    #[inline]
    pub fn SetOwnerID(&mut self, owner: c_int) {
        self.m_ownerID = owner;
    }

    #[inline]
    pub fn GetOwnerID(&self) -> c_int {
        self.m_ownerID
    }

    #[inline]
    pub fn GetInterface(&self) -> *mut interface_export_t {
        self.m_ie
    }

    #[inline]
    pub fn GetTaskManager(&self) -> *mut CTaskManager {
        self.m_taskManager
    }

    #[inline]
    pub fn SetTaskManager(&mut self, tm: *mut CTaskManager) {
        if !tm.is_null() {
            self.m_taskManager = tm;
        }
    }

    pub fn Save(&mut self) -> c_int {
        unimplemented!()
    }

    pub fn Load(&mut self) -> c_int {
        unimplemented!()
    }

    // moved to public on 2/12/2 to allow calling during shutdown
    pub fn Recall(&mut self) -> c_int {
        unimplemented!()
    }

    // Protected methods (private in Rust)

    fn EvaluateConditional(&mut self, block: *mut CBlock) -> c_int {
        unimplemented!()
    }

    fn Route(&mut self, sequence: *mut CSequence, bstream: *mut bstream_t) -> c_int {
        unimplemented!()
    }

    fn Flush(&mut self, owner: *mut CSequence) -> c_int {
        unimplemented!()
    }

    fn Interrupt(&mut self) {
        unimplemented!()
    }

    fn AddStream(&mut self) -> *mut bstream_t {
        unimplemented!()
    }

    fn DeleteStream(&mut self, bstream: *mut bstream_t) {
        unimplemented!()
    }

    fn AddAffect(&mut self, bstream: *mut bstream_t, retain: c_int, id: *mut c_int) -> c_int {
        unimplemented!()
    }

    fn AddSequence(&mut self) -> *mut CSequence {
        unimplemented!()
    }

    fn AddSequence_parent(
        &mut self,
        parent: *mut CSequence,
        returnSeq: *mut CSequence,
        flags: c_int,
    ) -> *mut CSequence {
        unimplemented!()
    }

    fn GetSequence(&self, id: c_int) -> *mut CSequence {
        unimplemented!()
    }

    // NOTENOTE: This only removes references to the sequence, IT DOES NOT FREE THE ALLOCATED MEMORY!
    fn RemoveSequence(&mut self, sequence: *mut CSequence) -> c_int {
        unimplemented!()
    }

    fn DestroySequence(&mut self, sequence: *mut CSequence) -> c_int {
        unimplemented!()
    }

    fn PushCommand(&mut self, command: *mut CBlock, flag: c_int) -> c_int {
        unimplemented!()
    }

    fn PopCommand(&mut self, flag: c_int) -> *mut CBlock {
        unimplemented!()
    }

    #[inline]
    fn ReturnSequence(&self, sequence: *mut CSequence) -> *mut CSequence {
        unimplemented!()
    }

    fn CheckRun(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn CheckLoop(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn CheckAffect(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn CheckIf(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn CheckDo(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn CheckFlush(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn Prep(&mut self, arg: *mut *mut CBlock) {
        unimplemented!()
    }

    fn Prime(&mut self, taskManager: *mut CTaskManager, command: *mut CBlock) -> c_int {
        unimplemented!()
    }

    fn StripExtension(&self, r#in: *const c_char, out: *mut c_char) {
        unimplemented!()
    }

    fn ParseRun(&mut self, block: *mut CBlock) -> c_int {
        unimplemented!()
    }

    fn ParseLoop(&mut self, block: *mut CBlock, bstream: *mut bstream_t) -> c_int {
        unimplemented!()
    }

    fn ParseAffect(&mut self, block: *mut CBlock, bstream: *mut bstream_t) -> c_int {
        unimplemented!()
    }

    fn ParseIf(&mut self, block: *mut CBlock, bstream: *mut bstream_t) -> c_int {
        unimplemented!()
    }

    fn ParseElse(&mut self, block: *mut CBlock, bstream: *mut bstream_t) -> c_int {
        unimplemented!()
    }

    fn ParseTask(&mut self, block: *mut CBlock, bstream: *mut bstream_t) -> c_int {
        unimplemented!()
    }

    fn Affect(&mut self, id: c_int, r#type: c_int) -> c_int {
        unimplemented!()
    }

    fn AddTaskSequence(&mut self, sequence: *mut CSequence, group: *mut CTaskGroup) {
        unimplemented!()
    }

    fn GetTaskSequence(&self, group: *mut CTaskGroup) -> *mut CSequence {
        unimplemented!()
    }
}
