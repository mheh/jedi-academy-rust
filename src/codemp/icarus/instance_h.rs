// ICARUS Intance header

#![allow(non_snake_case)]

use std::collections::HashMap;
use core::ffi::{c_int, c_char};

// Forward declarations for types from included headers

// From sequence.h
pub struct CSequence;

// From sequencer.h
pub struct CSequencer;

// From interface.h
pub struct interface_export_t;

pub struct ICARUS_Instance {
    pub m_interface: *mut interface_export_t,
    pub m_GUID: c_int,

    // sequence_l: typedef list< CSequence * >
    pub m_sequences: Vec<*mut CSequence>,

    // sequencer_l: typedef list< CSequencer * >
    pub m_sequencers: Vec<*mut CSequencer>,

    // signal_m: typedef map < string, unsigned char >
    pub m_signals: HashMap<String, u8>,

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
}

impl ICARUS_Instance {
    pub fn new() -> Self {
        todo!()
    }

    pub fn Create(iface: *mut interface_export_t) -> *mut ICARUS_Instance {
        todo!()
    }

    pub fn Delete(&mut self) -> c_int {
        todo!()
    }

    pub fn GetSequencer(&mut self, id: c_int) -> *mut CSequencer {
        todo!()
    }

    pub fn DeleteSequencer(&mut self, sequencer: *mut CSequencer) {
        todo!()
    }

    pub fn GetSequence(&mut self) -> *mut CSequence {
        todo!()
    }

    pub fn GetSequence_id(&mut self, id: c_int) -> *mut CSequence {
        todo!()
    }

    pub fn DeleteSequence(&mut self, sequence: *mut CSequence) {
        todo!()
    }

    pub fn GetInterface(&self) -> *mut interface_export_t {
        self.m_interface
    }

    // These are overriddable for "worst-case" save / loads
    // virtual int Save( void /*FIXME*/ );
    pub fn Save(&mut self) -> c_int {
        todo!()
    }

    // virtual int Load( void /*FIXME*/ );
    pub fn Load(&mut self) -> c_int {
        todo!()
    }

    pub fn Signal(&mut self, identifier: *const c_char) {
        todo!()
    }

    pub fn CheckSignal(&self, identifier: *const c_char) -> bool {
        todo!()
    }

    pub fn ClearSignal(&mut self, identifier: *const c_char) {
        todo!()
    }

    pub fn SaveSignals(&mut self) -> c_int {
        todo!()
    }

    pub fn SaveSequences(&mut self) -> c_int {
        todo!()
    }

    pub fn SaveSequenceIDTable(&mut self) -> c_int {
        todo!()
    }

    pub fn SaveSequencers(&mut self) -> c_int {
        todo!()
    }

    pub fn AllocateSequences(&mut self, numSequences: c_int, idTable: *mut c_int) -> c_int {
        todo!()
    }

    pub fn LoadSignals(&mut self) -> c_int {
        todo!()
    }

    pub fn LoadSequencers(&mut self) -> c_int {
        todo!()
    }

    pub fn LoadSequences(&mut self) -> c_int {
        todo!()
    }

    pub fn LoadSequence(&mut self) -> c_int {
        todo!()
    }

    pub fn Free(&mut self) -> c_int {
        todo!()
    }
}
