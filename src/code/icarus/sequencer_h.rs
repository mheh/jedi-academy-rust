// Sequencer Header File

#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};

// Forward declarations for external types
pub struct CBlockStream;
pub struct CSequence;
pub struct CTaskManager;
pub struct CBlock;
pub struct CIcarus;
pub struct CTaskGroup;
pub struct IGameInterface;

//Defines

//const int MAX_ERROR_LENGTH	= 256;

//Typedefs

#[repr(C)]
pub struct bstream_s
{
    pub stream: *mut CBlockStream,
    pub last: *mut bstream_s,
}

pub type bstream_t = bstream_s;

// Sequencer

/*
==================================================================================================

  CSequencer

==================================================================================================
*/

#[repr(C)]
pub struct CSequencer
{
    //typedef	map < int, CSequence * >			sequenceID_m;
    //typedef list < CSequence * >				sequence_l;
    //typedef map < CTaskGroup *, CSequence * >	taskSequence_m;

    // Member variables

    m_ownerID: c_int,

    m_taskManager: *mut CTaskManager,

    m_numCommands: c_int,		//Total number of commands for the sequencer (including all child sequences)

    //sequenceID_m		m_sequenceMap;
    //sequence_l			m_sequences;
    //taskSequence_m		m_taskSequences;

    m_curSequence: *mut CSequence,
    m_curGroup: *mut CTaskGroup,

    m_curStream: *mut bstream_t,

    m_elseValid: c_int,
    m_elseOwner: *mut CBlock,
    //vector<bstream_t*>  m_streamsCreated;

    m_id: c_int,
}

impl CSequencer
{
    // Enum constants
    pub const BF_ELSE: c_int = 0x00000001;	//Block has an else id	//FIXME: This was a sloppy fix for a problem that arose from conditionals

    pub const SEQ_OK: c_int = 0;				//Command was successfully added
    pub const SEQ_FAILED: c_int = 1;			//An error occured while trying to insert the command

    // Public methods

    pub fn GetID(&self) -> c_int { self.m_id }

    pub fn Init(&mut self, ownerID: c_int, taskManager: *mut CTaskManager) -> c_int {
        todo!()
    }

    pub fn Create() -> *mut CSequencer {
        todo!()
    }

    pub fn Free(&mut self, icarus: *mut CIcarus) {
        todo!()
    }

    pub fn Run(&mut self, buffer: *mut c_char, size: i64, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    pub fn Callback(&mut self, taskManager: *mut CTaskManager, block: *mut CBlock, returnCode: c_int, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    pub fn SetOwnerID(&mut self, owner: c_int) {	self.m_ownerID = owner; }

    pub fn GetOwnerID(&self) -> c_int { self.m_ownerID }

    pub fn GetTaskManager(&self) -> *mut CTaskManager { self.m_taskManager }

    pub fn SetTaskManager(&mut self, tm: *mut CTaskManager) { if !tm.is_null() { self.m_taskManager = tm; } }

    pub fn Save(&mut self) -> c_int {
        todo!()
    }

    pub fn Load(&mut self, icarus: *mut CIcarus, game: *mut IGameInterface) -> c_int {
        todo!()
    }

    // moved to public on 2/12/2 to allow calling during shutdown
    pub fn Recall(&mut self, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    // Protected methods

    fn EvaluateConditional(&mut self, block: *mut CBlock, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn Route(&mut self, sequence: *mut CSequence, bstream: *mut bstream_t, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn Flush(&mut self, owner: *mut CSequence, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn Interrupt(&mut self) {
        todo!()
    }

    fn AddStream(&mut self) -> *mut bstream_t {
        todo!()
    }

    fn DeleteStream(&mut self, bstream: *mut bstream_t) {
        todo!()
    }

    fn AddAffect(&mut self, bstream: *mut bstream_t, retain: c_int, id: *mut c_int, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn AddSequence(&mut self, icarus: *mut CIcarus) -> *mut CSequence {
        todo!()
    }

    fn AddSequenceWithParams(&mut self, parent: *mut CSequence, returnSeq: *mut CSequence, flags: c_int, icarus: *mut CIcarus) -> *mut CSequence {
        todo!()
    }

    fn GetSequence(&mut self, id: c_int) -> *mut CSequence {
        todo!()
    }

    //NOTENOTE: This only removes references to the sequence, IT DOES NOT FREE THE ALLOCATED MEMORY!
    fn RemoveSequence(&mut self, sequence: *mut CSequence, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn DestroySequence(&mut self, sequence: *mut CSequence, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn PushCommand(&mut self, command: *mut CBlock, flag: c_int) -> c_int {
        todo!()
    }

    fn PopCommand(&mut self, flag: c_int) -> *mut CBlock {
        todo!()
    }

    fn ReturnSequence(&mut self, sequence: *mut CSequence) -> *mut CSequence {
        todo!()
    }

    fn CheckRun(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn CheckLoop(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn CheckAffect(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn CheckIf(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn CheckDo(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn CheckFlush(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn Prep(&mut self, block: *mut *mut CBlock, icarus: *mut CIcarus) {
        todo!()
    }

    fn Prime(&mut self, taskManager: *mut CTaskManager, command: *mut CBlock, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn StripExtension(&mut self, input: *const c_char, output: *mut c_char) {
        todo!()
    }

    fn ParseRun(&mut self, block: *mut CBlock, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn ParseLoop(&mut self, block: *mut CBlock, bstream: *mut bstream_t, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn ParseAffect(&mut self, block: *mut CBlock, bstream: *mut bstream_t, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn ParseIf(&mut self, block: *mut CBlock, bstream: *mut bstream_t, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn ParseElse(&mut self, block: *mut CBlock, bstream: *mut bstream_t, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn ParseTask(&mut self, block: *mut CBlock, bstream: *mut bstream_t, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn Affect(&mut self, id: c_int, type_: c_int, icarus: *mut CIcarus) -> c_int {
        todo!()
    }

    fn AddTaskSequence(&mut self, sequence: *mut CSequence, group: *mut CTaskGroup) {
        todo!()
    }

    fn GetTaskSequence(&mut self, group: *mut CTaskGroup) -> *mut CSequence {
        todo!()
    }
}
