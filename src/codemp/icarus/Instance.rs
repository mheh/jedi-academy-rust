// Anything above this #include will be ignored by the compiler

// ICARUS Instance
//
//	-- jweier

#![allow(non_snake_case)]

use std::collections::HashMap;
use core::ffi::{c_int, c_char, c_void};
use std::ptr::{addr_of, addr_of_mut, null, null_mut};

use super::interface_h::{interface_export_t, interface_export_s};

// Forward declarations for types from included headers
pub struct CSequence;
pub struct CSequencer;
pub struct CTaskManager;

// We can't put these on entity fields since all that stuff is in C
// which can't be changed due to VMs. So we'll use a global array
// and access by the entity index given.
const MAX_GENTITIES: usize = 4096;

pub static mut gSequencers: [*mut CSequencer; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];
pub static mut gTaskManagers: [*mut CTaskManager; MAX_GENTITIES] = [null_mut(); MAX_GENTITIES];

const ICARUS_VERSION: f64 = 1.33;

#[allow(non_snake_case)]
pub struct ICARUS_Instance {
    pub m_GUID: c_int,
    pub m_interface: *mut interface_export_t,
    pub m_sequences: Vec<*mut CSequence>,
    pub m_sequencers: Vec<*mut CSequencer>,
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
    // Instance
    pub fn new() -> Self {
        ICARUS_Instance {
            m_GUID: 0,
            m_interface: null_mut(),
            m_sequences: Vec::new(),
            m_sequencers: Vec::new(),
            m_signals: HashMap::new(),

            #[cfg(debug_assertions)]
            m_DEBUG_NumSequencerAlloc: 0,
            #[cfg(debug_assertions)]
            m_DEBUG_NumSequencerFreed: 0,
            #[cfg(debug_assertions)]
            m_DEBUG_NumSequencerResidual: 0,

            #[cfg(debug_assertions)]
            m_DEBUG_NumSequenceAlloc: 0,
            #[cfg(debug_assertions)]
            m_DEBUG_NumSequenceFreed: 0,
            #[cfg(debug_assertions)]
            m_DEBUG_NumSequenceResidual: 0,
        }
    }

    // to be safe
    fn init_globals() {
        unsafe {
            // Zero out the global arrays
            core::ptr::write_bytes(addr_of_mut!(gSequencers) as *mut u8, 0, std::mem::size_of::<[*mut CSequencer; MAX_GENTITIES]>());
            core::ptr::write_bytes(addr_of_mut!(gTaskManagers) as *mut u8, 0, std::mem::size_of::<[*mut CTaskManager; MAX_GENTITIES]>());
        }
    }

    /*
    -------------------------
    Create
    -------------------------
    */
    pub fn Create(ie: *mut interface_export_t) -> *mut ICARUS_Instance {
        let mut instance = Box::new(ICARUS_Instance::new());
        instance.m_interface = ie;
        Self::init_globals();
        #[cfg(not(target_os = "linux"))]
        {
            // OutputDebugString( "ICARUS Instance successfully created\n" );
        }
        Box::into_raw(instance)
    }

    /*
    -------------------------
    Free
    -------------------------
    */
    pub fn Free(&mut self) -> c_int {
        // Delete any residual sequencers
        let sequencers_copy: Vec<_> = self.m_sequencers.clone();
        for sequencer in sequencers_copy {
            if !sequencer.is_null() {
                unsafe {
                    let _: Box<CSequencer> = Box::from_raw(sequencer);
                }
            }

            #[cfg(debug_assertions)]
            {
                self.m_DEBUG_NumSequencerResidual += 1;
            }
        }

        self.m_sequencers.clear();
        // all these are deleted now so clear the global map.
        unsafe {
            core::ptr::write_bytes(addr_of_mut!(gSequencers) as *mut u8, 0, std::mem::size_of::<[*mut CSequencer; MAX_GENTITIES]>());
            core::ptr::write_bytes(addr_of_mut!(gTaskManagers) as *mut u8, 0, std::mem::size_of::<[*mut CTaskManager; MAX_GENTITIES]>());
        }
        self.m_signals.clear();

        // Delete any residual sequences
        let sequences_copy: Vec<_> = self.m_sequences.clone();
        for sequence in sequences_copy {
            if !sequence.is_null() {
                unsafe {
                    let _: Box<CSequence> = Box::from_raw(sequence);
                }
            }

            #[cfg(debug_assertions)]
            {
                self.m_DEBUG_NumSequenceResidual += 1;
            }
        }

        self.m_sequences.clear();

        1
    }

    /*
    -------------------------
    Delete
    -------------------------
    */
    pub fn Delete(mut self_: Box<ICARUS_Instance>) -> c_int {
        let mut instance = self_;
        instance.Free();

        #[cfg(debug_assertions)]
        {
            let mut buffer = [0u8; 1024];

            // OutputDebugString( "\nICARUS Instance Debug Info:\n---------------------------\n" );

            // sprintf( (char *) buffer, "Sequencers Allocated:\t%d\n", m_DEBUG_NumSequencerAlloc );
            // OutputDebugString( (const char *) &buffer );

            // sprintf( (char *) buffer, "Sequencers Freed:\t\t%d\n", m_DEBUG_NumSequencerFreed );
            // OutputDebugString( (const char *) &buffer );

            // sprintf( (char *) buffer, "Sequencers Residual:\t%d\n\n", m_DEBUG_NumSequencerResidual );
            // OutputDebugString( (const char *) &buffer );

            // sprintf( (char *) buffer, "Sequences Allocated:\t%d\n", m_DEBUG_NumSequenceAlloc );
            // OutputDebugString( (const char *) &buffer );

            // sprintf( (char *) buffer, "Sequences Freed:\t\t%d\n", m_DEBUG_NumSequenceFreed );
            // OutputDebugString( (const char *) &buffer );

            // sprintf( (char *) buffer, "Sequences Residual:\t\t%d\n\n", m_DEBUG_NumSequenceResidual );
            // OutputDebugString( (const char *) &buffer );

            // OutputDebugString( "\n" );
        }

        drop(instance);

        1
    }

    /*
    -------------------------
    GetSequencer
    -------------------------
    */
    pub fn GetSequencer(&mut self, ownerID: c_int) -> *mut CSequencer {
        // This is a stub - CSequencer and CTaskManager creation would need their implementations
        // For now, returning null_mut() to indicate this is unimplemented
        null_mut()
    }

    /*
    -------------------------
    DeleteSequencer
    -------------------------
    */
    pub fn DeleteSequencer(&mut self, sequencer: *mut CSequencer) {
        // added 2/12/2 to properly delete blocks that were passed to the task manager
        // sequencer->Recall();

        // CTaskManager	*taskManager = sequencer->GetTaskManager();

        // if ( taskManager )
        // {
        //     taskManager->Free();
        //     delete taskManager;
        // }

        // Remove from vector
        self.m_sequencers.retain(|&s| s != sequencer);

        // sequencer->Free();
        if !sequencer.is_null() {
            unsafe {
                let _: Box<CSequencer> = Box::from_raw(sequencer);
            }
        }

        #[cfg(debug_assertions)]
        {
            self.m_DEBUG_NumSequencerFreed += 1;
        }
    }

    /*
    -------------------------
    GetSequence
    -------------------------
    */
    pub fn GetSequence(&mut self) -> *mut CSequence {
        // This is a stub - CSequence creation would need its implementation
        // For now, returning null_mut() to indicate this is unimplemented
        null_mut()
    }

    /*
    -------------------------
    GetSequence
    -------------------------
    */
    pub fn GetSequence_id(&mut self, id: c_int) -> *mut CSequence {
        for seq in &self.m_sequences {
            if !seq.is_null() {
                // This would need access to CSequence::GetID()
                // For now, this is a stub
            }
        }

        null_mut()
    }

    /*
    -------------------------
    DeleteSequence
    -------------------------
    */
    pub fn DeleteSequence(&mut self, sequence: *mut CSequence) {
        self.m_sequences.retain(|&s| s != sequence);

        if !sequence.is_null() {
            unsafe {
                let _: Box<CSequence> = Box::from_raw(sequence);
            }
        }

        #[cfg(debug_assertions)]
        {
            self.m_DEBUG_NumSequenceFreed += 1;
        }
    }

    /*
    -------------------------
    AllocateSequences
    -------------------------
    */
    pub fn AllocateSequences(&mut self, numSequences: c_int, idTable: *mut c_int) -> c_int {
        if idTable.is_null() {
            return 0;
        }

        for i in 0..numSequences as usize {
            // If the GUID of this sequence is higher than the current, take this a the "current" GUID
            unsafe {
                let id = *idTable.add(i);
                if id > self.m_GUID {
                    self.m_GUID = id;
                }
            }

            // Allocate the container sequence
            // This is a stub - would need actual CSequence creation
            // if ( ( sequence = GetSequence() ) == NULL )
            //     return false;

            // Override the given GUID with the real one
            // sequence->SetID( idTable[i] );
        }

        1
    }

    /*
    -------------------------
    SaveSequenceIDTable
    -------------------------
    */
    pub fn SaveSequenceIDTable(&mut self) -> c_int {
        // Save out the number of sequences to follow
        let numSequences = self.m_sequences.len() as c_int;
        unsafe {
            if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                write_fn(
                    0x23534551u32 as c_int as u32 as u32,  // '#SEQ'
                    addr_of_mut!(numSequences as c_int) as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                );
            }
        }

        // Sequences are saved first, by ID and information

        // First pass, save all sequences ID for reconstruction
        let mut idTable: Vec<c_int> = Vec::with_capacity(numSequences as usize);
        for _ in 0..numSequences {
            idTable.push(0);
        }

        let mut itr = 0;
        for sequencer in &self.m_sequencers {
            if !sequencer.is_null() && itr < idTable.len() {
                // idTable[itr++] = (*sqi)->GetID();
                itr += 1;
            }
        }

        unsafe {
            if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                write_fn(
                    0x42545153u32 as c_int as u32 as u32,  // 'SQTB'
                    idTable.as_mut_ptr() as *mut c_void,
                    (std::mem::size_of::<c_int>() as c_int) * numSequences,
                );
            }
        }

        1
    }

    /*
    -------------------------
    SaveSequences
    -------------------------
    */
    pub fn SaveSequences(&mut self) -> c_int {
        // Save out a listing of all the used sequences by ID
        self.SaveSequenceIDTable();

        // Save all the information in order
        for sequence in &self.m_sequences {
            if !sequence.is_null() {
                // (*sqi)->Save();
            }
        }

        1
    }

    /*
    -------------------------
    SaveSequencers
    -------------------------
    */
    pub fn SaveSequencers(&mut self) -> c_int {
        // Save out the number of sequences to follow
        let numSequencers = self.m_sequencers.len() as c_int;
        unsafe {
            if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                write_fn(
                    0x52515323u32 as c_int as u32 as u32,  // '#SQR'
                    addr_of_mut!(numSequencers as c_int) as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                );
            }
        }

        // The sequencers are then saved
        for sequencer in &self.m_sequencers {
            if !sequencer.is_null() {
                // (*si)->Save();
            }
        }

        1
    }

    /*
    -------------------------
    SaveSignals
    -------------------------
    */
    pub fn SaveSignals(&mut self) -> c_int {
        let numSignals = self.m_signals.len() as c_int;

        unsafe {
            if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                write_fn(
                    0x47495349u32 as c_int as u32 as u32,  // 'ISIG'
                    addr_of_mut!(numSignals as c_int) as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                );
            }
        }

        for (name, _) in &self.m_signals {
            let name_bytes = name.as_bytes();
            let name_cstr = std::ffi::CString::new(name_bytes).unwrap_or_default();
            let name_ptr = name_cstr.as_ptr();

            // Make sure this is a valid string
            assert!(!name_ptr.is_null() && unsafe { *name_ptr } as u8 != 0);

            let length = (name_bytes.len() + 1) as c_int;

            // Save out the string size
            unsafe {
                if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                    write_fn(
                        0x23474953u32 as c_int as u32 as u32,  // 'SIG#'
                        addr_of_mut!(length as c_int) as *mut c_void,
                        std::mem::size_of::<c_int>() as c_int,
                    );
                }
            }

            // Write out the string
            unsafe {
                if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                    write_fn(
                        0x4e474953u32 as c_int as u32 as u32,  // 'SIGN'
                        name_ptr as *mut c_void,
                        length,
                    );
                }
            }
        }

        1
    }

    /*
    -------------------------
    Save
    -------------------------
    */
    pub fn Save(&mut self) -> c_int {
        // Save out a ICARUS save block header with the ICARUS version
        let version = ICARUS_VERSION;
        unsafe {
            if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                write_fn(
                    0x52414349u32 as c_int as u32 as u32,  // 'ICAR'
                    addr_of_mut!(version as f64) as *mut c_void,
                    std::mem::size_of::<f64>() as c_int,
                );
            }
        }

        // Save out the signals
        if self.SaveSignals() == 0 {
            return 0;
        }

        // Save out the sequences
        if self.SaveSequences() == 0 {
            return 0;
        }

        // Save out the sequencers
        if self.SaveSequencers() == 0 {
            return 0;
        }

        let version = ICARUS_VERSION;
        unsafe {
            if let Some(write_fn) = (*self.m_interface).I_WriteSaveData {
                write_fn(
                    0x444e4549u32 as c_int as u32 as u32,  // 'IEND'
                    addr_of_mut!(version as f64) as *mut c_void,
                    std::mem::size_of::<f64>() as c_int,
                );
            }
        }

        1
    }

    /*
    -------------------------
    LoadSignals
    -------------------------
    */
    pub fn LoadSignals(&mut self) -> c_int {
        let mut numSignals = 0c_int;

        unsafe {
            if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                read_fn(
                    0x47495349u32 as c_int as u32 as u32,  // 'ISIG'
                    addr_of_mut!(numSignals) as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                );
            }
        }

        for _i in 0..numSignals {
            let mut buffer = [0u8; 1024];
            let mut length = 0c_int;

            // Get the size of the string
            unsafe {
                if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                    read_fn(
                        0x23474953u32 as c_int as u32 as u32,  // 'SIG#'
                        addr_of_mut!(length) as *mut c_void,
                        std::mem::size_of::<c_int>() as c_int,
                    );
                }
            }

            assert!((length as usize) < buffer.len());

            // Get the string
            unsafe {
                if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                    read_fn(
                        0x4e474953u32 as c_int as u32 as u32,  // 'SIGN'
                        buffer.as_mut_ptr() as *mut c_void,
                        length,
                    );
                }
            }

            // Turn it on and add it to the system
            if let Ok(s) = std::ffi::CStr::from_bytes_until_nul(&buffer) {
                if let Ok(signal_name) = s.to_str() {
                    self.Signal(signal_name.as_ptr() as *const c_char);
                }
            }
        }

        1
    }

    /*
    -------------------------
    LoadSequence
    -------------------------
    */
    pub fn LoadSequence(&mut self) -> c_int {
        // CSequence	*sequence = GetSequence();

        // Load the sequence back in
        // sequence->Load();

        // If this sequence had a higher GUID than the current, save it
        // if ( sequence->GetID() > m_GUID )
        //     m_GUID = sequence->GetID();

        1
    }

    /*
    -------------------------
    LoadSequence
    -------------------------
    */
    pub fn LoadSequences(&mut self) -> c_int {
        let mut numSequences = 0c_int;

        // Get the number of sequences to read in
        unsafe {
            if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                read_fn(
                    0x51453223u32 as c_int as u32 as u32,  // '#SEQ'
                    addr_of_mut!(numSequences) as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                );
            }
        }

        let mut idTable: Vec<c_int> = Vec::with_capacity(numSequences as usize);
        for _ in 0..numSequences {
            idTable.push(0);
        }

        if idTable.is_empty() && numSequences > 0 {
            return 0;
        }

        // Load the sequencer ID table
        unsafe {
            if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                read_fn(
                    0x42545153u32 as c_int as u32 as u32,  // 'SQTB'
                    idTable.as_mut_ptr() as *mut c_void,
                    (std::mem::size_of::<c_int>() as c_int) * numSequences,
                );
            }
        }

        // First pass, allocate all container sequences and give them their proper IDs
        if self.AllocateSequences(numSequences, idTable.as_mut_ptr()) == 0 {
            return 0;
        }

        // Second pass, load all sequences
        for i in 0..numSequences as usize {
            // Get the proper sequence for this load
            // if ( ( sequence = GetSequence( idTable[i] ) ) == NULL )
            //     return false;

            // Load the sequence
            // if ( ( sequence->Load() ) == false )
            //     return false;
        }

        1
    }

    /*
    -------------------------
    LoadSequencers
    -------------------------
    */
    pub fn LoadSequencers(&mut self) -> c_int {
        let mut numSequencers = 0c_int;

        // Get the number of sequencers to load
        unsafe {
            if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                read_fn(
                    0x52515323u32 as c_int as u32 as u32,  // '#SQR'
                    addr_of_mut!(numSequencers) as *mut c_void,
                    std::mem::size_of::<c_int>() as c_int,
                );
            }
        }

        // Load all sequencers
        for _i in 0..numSequencers {
            // NOTENOTE: The ownerID will be replaced in the loading process
            // if ( ( sequencer = GetSequencer( -1 ) ) == NULL )
            //     return false;

            // if ( sequencer->Load() == false )
            //     return false;
        }

        1
    }

    /*
    -------------------------
    Load
    -------------------------
    */
    pub fn Load(&mut self) -> c_int {
        // Clear out any old information
        self.Free();

        // Check to make sure we're at the ICARUS save block
        let mut version = 0f64;
        unsafe {
            if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                read_fn(
                    0x52414349u32 as c_int as u32 as u32,  // 'ICAR'
                    addr_of_mut!(version) as *mut c_void,
                    std::mem::size_of::<f64>() as c_int,
                );
            }
        }

        // Versions must match!
        if (version - ICARUS_VERSION).abs() > f64::EPSILON {
            unsafe {
                if let Some(printf_fn) = (*self.m_interface).I_DPrintf {
                    // printf_fn( WL_ERROR, "save game data contains outdated ICARUS version information!\n");
                }
            }
            return 0;
        }

        // Load all signals
        if self.LoadSignals() == 0 {
            unsafe {
                if let Some(printf_fn) = (*self.m_interface).I_DPrintf {
                    // printf_fn( WL_ERROR, "failed to load signals from save game!\n");
                }
            }
            return 0;
        }

        // Load in all sequences
        if self.LoadSequences() == 0 {
            unsafe {
                if let Some(printf_fn) = (*self.m_interface).I_DPrintf {
                    // printf_fn( WL_ERROR, "failed to load sequences from save game!\n");
                }
            }
            return 0;
        }

        // Load in all sequencers
        if self.LoadSequencers() == 0 {
            unsafe {
                if let Some(printf_fn) = (*self.m_interface).I_DPrintf {
                    // printf_fn( WL_ERROR, "failed to load sequencers from save game!\n");
                }
            }
            return 0;
        }

        let mut version = 0f64;
        unsafe {
            if let Some(read_fn) = (*self.m_interface).I_ReadSaveData {
                read_fn(
                    0x444e4549u32 as c_int as u32 as u32,  // 'IEND'
                    addr_of_mut!(version) as *mut c_void,
                    std::mem::size_of::<f64>() as c_int,
                );
            }
        }

        1
    }

    /*
    -------------------------
    Signal
    -------------------------
    */
    pub fn Signal(&mut self, identifier: *const c_char) {
        if identifier.is_null() {
            return;
        }
        unsafe {
            if let Ok(s) = std::ffi::CStr::from_ptr(identifier).to_str() {
                self.m_signals.insert(s.to_string(), 1);
            }
        }
    }

    /*
    -------------------------
    CheckSignal
    -------------------------
    */
    pub fn CheckSignal(&self, identifier: *const c_char) -> bool {
        if identifier.is_null() {
            return false;
        }
        unsafe {
            if let Ok(s) = std::ffi::CStr::from_ptr(identifier).to_str() {
                return self.m_signals.contains_key(s);
            }
        }
        false
    }

    /*
    -------------------------
    ClearSignal
    -------------------------
    */
    pub fn ClearSignal(&mut self, identifier: *const c_char) {
        if identifier.is_null() {
            return;
        }
        unsafe {
            if let Ok(s) = std::ffi::CStr::from_ptr(identifier).to_str() {
                self.m_signals.remove(s);
            }
        }
    }
}
