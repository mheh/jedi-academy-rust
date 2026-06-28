// IcarusImplementation.rs
#![allow(non_snake_case)]

use core::ffi::{c_int, c_char, c_void};
use std::collections::HashMap;
use std::ptr;
use std::ffi::CStr;

// Forward declarations from other modules
pub struct CBlockStream;
pub struct CSequence;
pub struct CTaskManager;
pub struct CSequencer;
pub struct CBlockMember;
pub struct CBlock;
pub struct IGameInterface;

// Type aliases for STL containers
// signal_m is std::map<std::string, int>
type signal_m = HashMap<String, i32>;

// sequencer_l is std::list<CSequencer*>
type sequencer_l = Vec<*mut CSequencer>;

// sequence_l is std::list<CSequence*>
type sequence_l = Vec<*mut CSequence>;

// sequencer_m is std::map<int, CSequencer*>
type sequencer_m = HashMap<c_int, *mut CSequencer>;

// Macro translations
// #define STL_ITERATE( a, b )		for ( a = b.begin(); a != b.end(); a++ )
// Translated to Rust: for item in &collection { ... }

// #define STL_INSERT( a, b )		a.insert( a.end(), b );
// Translated to Rust: a.push(b);

//////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// required implementation of CIcarusInterface

pub struct IIcarusInterface;

impl IIcarusInterface {
    // IIcarusInterface* IIcarusInterface::GetIcarus(int flavor,bool constructIfNecessary)
    pub fn GetIcarus(flavor: c_int, constructIfNecessary: bool) -> *mut IIcarusInterface {
        unsafe {
            if S_INSTANCES.is_null() && constructIfNecessary {
                S_FLAVORS_AVAILABLE = IGameInterface_s_IcarusFlavorsNeeded;
                if S_FLAVORS_AVAILABLE == 0 {
                    return ptr::null_mut();
                }
                S_INSTANCES = libc::malloc((S_FLAVORS_AVAILABLE as usize) * std::mem::size_of::<*mut CIcarus>()) as *mut *mut CIcarus;
                for index in 0..S_FLAVORS_AVAILABLE {
                    let icarus_ptr = CIcarus_new(index);
                    *S_INSTANCES.add(index as usize) = icarus_ptr;
                    // OutputDebugString( "ICARUS flavor successfully created\n" );
                }
            }

            if flavor >= S_FLAVORS_AVAILABLE || S_INSTANCES.is_null() {
                return ptr::null_mut();
            }
            return *S_INSTANCES.add(flavor as usize) as *mut IIcarusInterface;
        }
    }

    // void IIcarusInterface::DestroyIcarus()
    pub fn DestroyIcarus() {
        unsafe {
            for index in 0..S_FLAVORS_AVAILABLE {
                let icarus_ptr = *S_INSTANCES.add(index as usize);
                libc::free(icarus_ptr as *mut c_void);
            }
            libc::free(S_INSTANCES as *mut c_void);
            S_INSTANCES = ptr::null_mut();
            S_FLAVORS_AVAILABLE = 0;
        }
    }

    // IIcarusInterface::~IIcarusInterface()
    pub fn destructor(&mut self) {
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////
//
// CIcarus

pub struct CIcarus {
    pub m_flavor: c_int,
    pub m_nextSequencerID: c_int,
    pub m_GUID: c_int,

    pub m_sequences: sequence_l,
    pub m_sequencers: sequencer_l,
    pub m_sequencerMap: sequencer_m,

    pub m_signals: signal_m,

    // DEBUG members (conditional compilation handled below)
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

pub static mut ICARUS_VERSION: f64 = 1.40;

pub static mut S_FLAVORS_AVAILABLE: c_int = 0;

pub static mut S_INSTANCES: *mut *mut CIcarus = ptr::null_mut();

// Stub for game interface global
pub static mut IGameInterface_s_IcarusFlavorsNeeded: c_int = 0;

// CIcarus::CIcarus(int flavor) :
//     m_flavor(flavor), m_nextSequencerID(0)
pub fn CIcarus_new(flavor: c_int) -> *mut CIcarus {
    let icarus = Box::new(CIcarus {
        m_flavor: flavor,
        m_nextSequencerID: 0,
        m_GUID: 0,

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

        m_ulBufferCurPos: 0,
        m_ulBytesRead: 0,
        m_byBuffer: ptr::null_mut(),

        m_sequences: Vec::new(),
        m_sequencers: Vec::new(),
        m_sequencerMap: HashMap::new(),
        m_signals: HashMap::new(),
    });

    Box::into_raw(icarus)
}

// CIcarus::~CIcarus()
pub fn CIcarus_delete(icarus: *mut CIcarus) {
    unsafe {
        if !icarus.is_null() {
            CIcarus_Delete(icarus);
            Box::from_raw(icarus);
        }
    }
}

#[cfg(all(debug_assertions, target_os = "windows"))]
extern "C" {
    // for OutputDebugString on Windows debug builds
    fn OutputDebugString(lpOutputString: *const u8);
}

// For non-Windows or non-debug builds, provide a stub
#[cfg(not(all(debug_assertions, target_os = "windows")))]
fn OutputDebugString(_lpOutputString: *const u8) {
}

// void CIcarus::Delete( void )
pub fn CIcarus_Delete(icarus: *mut CIcarus) {
    unsafe {
        if icarus.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        CIcarus_Free(icarus);

        #[cfg(debug_assertions)]
        {
            let mut buffer: [u8; 1024] = [0; 1024];

            // OutputDebugString( "\nICARUS Instance Debug Info:\n---------------------------\n" );
            let header_msg = b"\nICARUS Instance Debug Info:\n---------------------------\n\0";
            OutputDebugString(header_msg as *const u8);

            let len = libc::snprintf(
                buffer.as_mut_ptr() as *mut i8,
                1024,
                "Sequencers Allocated:\t%d\n\0".as_ptr() as *const i8,
                icarus_ref.m_DEBUG_NumSequencerAlloc,
            );
            OutputDebugString(buffer.as_ptr());

            let len = libc::snprintf(
                buffer.as_mut_ptr() as *mut i8,
                1024,
                "Sequencers Freed:\t\t%d\n\0".as_ptr() as *const i8,
                icarus_ref.m_DEBUG_NumSequencerFreed,
            );
            OutputDebugString(buffer.as_ptr());

            let len = libc::snprintf(
                buffer.as_mut_ptr() as *mut i8,
                1024,
                "Sequencers Residual:\t%d\n\n\0".as_ptr() as *const i8,
                icarus_ref.m_DEBUG_NumSequencerResidual,
            );
            OutputDebugString(buffer.as_ptr());

            let len = libc::snprintf(
                buffer.as_mut_ptr() as *mut i8,
                1024,
                "Sequences Allocated:\t%d\n\0".as_ptr() as *const i8,
                icarus_ref.m_DEBUG_NumSequenceAlloc,
            );
            OutputDebugString(buffer.as_ptr());

            let len = libc::snprintf(
                buffer.as_mut_ptr() as *mut i8,
                1024,
                "Sequences Freed:\t\t%d\n\0".as_ptr() as *const i8,
                icarus_ref.m_DEBUG_NumSequenceFreed,
            );
            OutputDebugString(buffer.as_ptr());

            let len = libc::snprintf(
                buffer.as_mut_ptr() as *mut i8,
                1024,
                "Sequences Residual:\t\t%d\n\n\0".as_ptr() as *const i8,
                icarus_ref.m_DEBUG_NumSequenceResidual,
            );
            OutputDebugString(buffer.as_ptr());

            let newline_msg = b"\n\0";
            OutputDebugString(newline_msg as *const u8);
        }
    }
}

// void CIcarus::Signal( const char *identifier )
pub fn CIcarus_Signal(icarus: *mut CIcarus, identifier: *const c_char) {
    unsafe {
        if icarus.is_null() || identifier.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;
        let ident_str = CStr::from_ptr(identifier)
            .to_string_lossy()
            .into_owned();
        icarus_ref.m_signals.insert(ident_str, 1);
    }
}

// bool CIcarus::CheckSignal( const char *identifier )
pub fn CIcarus_CheckSignal(icarus: *const CIcarus, identifier: *const c_char) -> bool {
    unsafe {
        if icarus.is_null() || identifier.is_null() {
            return false;
        }
        let icarus_ref = &*icarus;
        let ident_str = CStr::from_ptr(identifier).to_string_lossy();

        for (key, _) in icarus_ref.m_signals.iter() {
            if key == ident_str.as_ref() {
                return true;
            }
        }
        false
    }
}

// void CIcarus::ClearSignal( const char *identifier )
pub fn CIcarus_ClearSignal(icarus: *mut CIcarus, identifier: *const c_char) {
    unsafe {
        if icarus.is_null() || identifier.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;
        let ident_str = CStr::from_ptr(identifier)
            .to_string_lossy()
            .into_owned();
        icarus_ref.m_signals.remove(&ident_str);
    }
}

// void CIcarus::Free( void )
pub fn CIcarus_Free(icarus: *mut CIcarus) {
    unsafe {
        if icarus.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        // Delete any residual sequencers
        for sri in icarus_ref.m_sequencers.iter() {
            CSequencer_Free(*sri, icarus);

            #[cfg(debug_assertions)]
            {
                icarus_ref.m_DEBUG_NumSequencerResidual += 1;
            }
        }

        icarus_ref.m_sequencers.clear();
        icarus_ref.m_signals.clear();

        // Delete any residual sequences
        for si in icarus_ref.m_sequences.iter() {
            CSequence_Delete(*si, icarus);
            libc::free(*si as *mut c_void);

            #[cfg(debug_assertions)]
            {
                icarus_ref.m_DEBUG_NumSequenceResidual += 1;
            }
        }

        icarus_ref.m_sequences.clear();
        icarus_ref.m_sequencerMap.clear();
    }
}

// int CIcarus::GetIcarusID( int gameID )
pub fn CIcarus_GetIcarusID(icarus: *mut CIcarus, gameID: c_int) -> c_int {
    unsafe {
        if icarus.is_null() {
            return -1;
        }
        let icarus_ref = &mut *icarus;

        let sequencer = CSequencer_Create();
        let taskManager = CTaskManager_Create();

        if sequencer.is_null() || taskManager.is_null() {
            return -1;
        }

        CSequencer_Init(sequencer, gameID, taskManager);
        CTaskManager_Init(taskManager, sequencer);

        icarus_ref.m_sequencers.push(sequencer);

        let sequencer_id = CSequencer_GetID(sequencer);
        icarus_ref.m_sequencerMap.insert(sequencer_id, sequencer);

        #[cfg(debug_assertions)]
        {
            icarus_ref.m_DEBUG_NumSequencerAlloc += 1;
        }

        sequencer_id
    }
}

// void CIcarus::DeleteIcarusID( int& icarusID )
pub fn CIcarus_DeleteIcarusID(icarus: *mut CIcarus, icarusID: &mut c_int) {
    unsafe {
        if icarus.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        let sequencer = CIcarus_FindSequencer(icarus, *icarusID);
        if sequencer.is_null() {
            *icarusID = -1;
            return;
        }

        let taskManager = CSequencer_GetTaskManager(sequencer);
        if CTaskManager_IsResident(taskManager) {
            let game = IGameInterface_GetGame(icarus_ref.m_flavor);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "Refusing DeleteIcarusID(%d) because it is running!\n\0".as_ptr() as *const i8, *icarusID);
            }
            assert!(false);
            return;
        }

        icarus_ref.m_sequencerMap.remove(icarusID);

        // added 2/12/2 to properly delete blocks that were passed to the task manager
        CSequencer_Recall(sequencer, icarus);

        if !taskManager.is_null() {
            CTaskManager_Free(taskManager);
            libc::free(taskManager as *mut c_void);
        }

        // Remove from sequencers list (simulating std::list::remove)
        icarus_ref.m_sequencers.retain(|&x| x != sequencer);

        CSequencer_Free(sequencer, icarus);

        #[cfg(debug_assertions)]
        {
            icarus_ref.m_DEBUG_NumSequencerFreed += 1;
        }

        *icarusID = -1;
    }
}

// CSequence *CIcarus::GetSequence( void )
pub fn CIcarus_GetSequence(icarus: *mut CIcarus) -> *mut CSequence {
    unsafe {
        if icarus.is_null() {
            return ptr::null_mut();
        }
        let icarus_ref = &mut *icarus;

        let sequence = CSequence_Create();

        // Assign the GUID
        CSequence_SetID(sequence, icarus_ref.m_GUID);
        icarus_ref.m_GUID += 1;

        icarus_ref.m_sequences.push(sequence);

        #[cfg(debug_assertions)]
        {
            icarus_ref.m_DEBUG_NumSequenceAlloc += 1;
        }

        sequence
    }
}

// CSequence *CIcarus::GetSequence( int id )
pub fn CIcarus_GetSequence_ById(icarus: *const CIcarus, id: c_int) -> *mut CSequence {
    unsafe {
        if icarus.is_null() {
            return ptr::null_mut();
        }
        let icarus_ref = &*icarus;

        for si in icarus_ref.m_sequences.iter() {
            if CSequence_GetID(*si) == id {
                return *si;
            }
        }

        ptr::null_mut()
    }
}

// void CIcarus::DeleteSequence( CSequence *sequence )
pub fn CIcarus_DeleteSequence(icarus: *mut CIcarus, sequence: *mut CSequence) {
    unsafe {
        if icarus.is_null() || sequence.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        icarus_ref.m_sequences.retain(|&x| x != sequence);

        CSequence_Delete(sequence, icarus);
        libc::free(sequence as *mut c_void);

        #[cfg(debug_assertions)]
        {
            icarus_ref.m_DEBUG_NumSequenceFreed += 1;
        }
    }
}

// int CIcarus::AllocateSequences( int numSequences, int *idTable )
pub fn CIcarus_AllocateSequences(icarus: *mut CIcarus, numSequences: c_int, idTable: *mut c_int) -> c_int {
    unsafe {
        if icarus.is_null() || idTable.is_null() {
            return 0;
        }
        let icarus_ref = &mut *icarus;

        for i in 0..numSequences {
            // If the GUID of this sequence is higher than the current, take this a the "current" GUID
            if *idTable.add(i as usize) > icarus_ref.m_GUID {
                icarus_ref.m_GUID = *idTable.add(i as usize);
            }

            // Allocate the container sequence
            let sequence = CIcarus_GetSequence(icarus);
            if sequence.is_null() {
                return 0;
            }

            // Override the given GUID with the real one
            CSequence_SetID(sequence, *idTable.add(i as usize));
        }

        1
    }
}

// void CIcarus::Precache(char* buffer, long length)
pub fn CIcarus_Precache(icarus: *mut CIcarus, buffer: *mut c_char, length: i64) {
    unsafe {
        if icarus.is_null() || buffer.is_null() {
            return;
        }
        let icarus_ref = &*icarus;

        let game = IGameInterface_GetGame(icarus_ref.m_flavor);
        let mut stream: CBlockStream = std::mem::zeroed(); // LOCAL STUB: CBlockStream

        if CBlockStream_Open(&mut stream, buffer, length) == 0 {
            return;
        }

        let mut sVal1: *const c_char = ptr::null();
        let mut sVal2: *const c_char = ptr::null();

        // Now iterate through all blocks of the script, searching for keywords
        while CBlockStream_BlockAvailable(&stream) != 0 {
            // Get a block
            let mut block: CBlock = std::mem::zeroed(); // LOCAL STUB: CBlock
            if CBlockStream_ReadBlock(&mut stream, &mut block, icarus as *mut c_void) == 0 {
                return;
            }

            // Determine what type of block this is
            let block_id = CBlock_GetBlockID(&block);
            match block_id {
                22 => { // ID_CAMERA: to cache ROFF files
                    let f = *(CBlock_GetMemberData(&block, 0) as *const f32);

                    if f == 10.0 { // TYPE_PATH (from header defines)
                        sVal1 = CBlock_GetMemberData(&block, 1) as *const c_char;

                        if !game.is_null() {
                            IGameInterface_PrecacheRoff(game, sVal1);
                        }
                    }
                }
                29 => { // ID_PLAY: to cache ROFF files
                    sVal1 = CBlock_GetMemberData(&block, 0) as *const c_char;

                    if strcmp_safe(sVal1, "PLAY_ROFF\0".as_ptr() as *const c_char) == 0 {
                        sVal1 = CBlock_GetMemberData(&block, 1) as *const c_char;

                        if !game.is_null() {
                            IGameInterface_PrecacheRoff(game, sVal1);
                        }
                    }
                }
                // Run commands
                13 => { // ID_RUN
                    sVal1 = CBlock_GetMemberData(&block, 0) as *const c_char;
                    if !game.is_null() {
                        IGameInterface_PrecacheScript(game, sVal1);
                    }
                }
                1 => { // ID_SOUND
                    sVal1 = CBlock_GetMemberData(&block, 1) as *const c_char; // 0 is channel, 1 is filename
                    if !game.is_null() {
                        IGameInterface_PrecacheSound(game, sVal1);
                    }
                }
                7 => { // ID_SET
                    let blockMember = CBlock_GetMember(&block, 0);

                    // NOTENOTE: This will not catch special case get() inlines! (There's not really a good way to do that)

                    // Make sure we're testing against strings
                    if CBlockMember_GetID(blockMember) == 4 { // TK_STRING
                        sVal1 = CBlock_GetMemberData(&block, 0) as *const c_char;
                        sVal2 = CBlock_GetMemberData(&block, 1) as *const c_char;

                        if !game.is_null() {
                            IGameInterface_PrecacheFromSet(game, sVal1, sVal2);
                        }
                    }
                }
                _ => {}
            }

            // Clean out the block for the next pass
            CBlock_Free(&mut block, icarus as *mut c_void);
        }

        // All done
        CBlockStream_Free(&mut stream);
    }
}

// CSequencer* CIcarus::FindSequencer(int sequencerID)
pub fn CIcarus_FindSequencer(icarus: *const CIcarus, sequencerID: c_int) -> *mut CSequencer {
    unsafe {
        if icarus.is_null() {
            return ptr::null_mut();
        }
        let icarus_ref = &*icarus;

        if let Some(&sequencer) = icarus_ref.m_sequencerMap.get(&sequencerID) {
            return sequencer;
        }

        ptr::null_mut()
    }
}

// int CIcarus::Run(int icarusID, char* buffer, long length)
pub fn CIcarus_Run(icarus: *mut CIcarus, icarusID: c_int, buffer: *mut c_char, length: i64) -> c_int {
    unsafe {
        if icarus.is_null() {
            return -1;
        }

        let sequencer = CIcarus_FindSequencer(icarus, icarusID);
        if !sequencer.is_null() {
            return CSequencer_Run(sequencer, buffer, length, icarus);
        }
        -1
    }
}

// int CIcarus::SaveSequenceIDTable()
pub fn CIcarus_SaveSequenceIDTable(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }
        let icarus_ref = &mut *icarus;

        // Save out the number of sequences to follow
        let numSequences = icarus_ref.m_sequences.len() as c_int;

        CIcarus_BufferWrite(icarus, &numSequences as *const c_int as *const c_void, std::mem::size_of::<c_int>() as u64);

        // Sequences are saved first, by ID and information
        // First pass, save all sequences ID for reconstruction
        let idTable = libc::malloc((numSequences as usize) * std::mem::size_of::<c_int>()) as *mut c_int;
        if idTable.is_null() {
            return 0;
        }

        let mut itr = 0;
        for sqi in icarus_ref.m_sequences.iter() {
            *idTable.add(itr as usize) = CSequence_GetID(*sqi);
            itr += 1;
        }

        CIcarus_BufferWrite(icarus, idTable as *const c_void, (std::mem::size_of::<c_int>() as c_int * numSequences) as u64);

        libc::free(idTable as *mut c_void);

        1
    }
}

// int CIcarus::SaveSequences()
pub fn CIcarus_SaveSequences(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }

        // Save out a listing of all the used sequences by ID
        CIcarus_SaveSequenceIDTable(icarus);

        let icarus_ref = &*icarus;
        // Save all the information in order
        for sqi in icarus_ref.m_sequences.iter() {
            CSequence_Save(*sqi);
        }

        1
    }
}

// int CIcarus::SaveSequencers()
pub fn CIcarus_SaveSequencers(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }
        let icarus_ref = &mut *icarus;

        // Save out the number of sequences to follow
        let numSequencers = icarus_ref.m_sequencers.len() as c_int;
        CIcarus_BufferWrite(icarus, &numSequencers as *const c_int as *const c_void, std::mem::size_of::<c_int>() as u64);

        // The sequencers are then saved
        let mut sequencessaved = 0;
        for si in icarus_ref.m_sequencers.iter() {
            CSequencer_Save(*si);
            sequencessaved += 1;
        }

        assert!(sequencessaved == numSequencers);

        1
    }
}

// int CIcarus::SaveSignals()
pub fn CIcarus_SaveSignals(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }
        let icarus_ref = &mut *icarus;

        let numSignals = icarus_ref.m_signals.len() as c_int;

        CIcarus_BufferWrite(icarus, &numSignals as *const c_int as *const c_void, std::mem::size_of::<c_int>() as u64);

        for (name, _) in icarus_ref.m_signals.iter() {
            let c_name = std::ffi::CString::new(name.clone()).unwrap();
            let name_ptr = c_name.as_ptr();
            let length = (libc::strlen(name_ptr) + 1) as c_int;

            // Save out the string size
            CIcarus_BufferWrite(icarus, &length as *const c_int as *const c_void, std::mem::size_of::<c_int>() as u64);

            // Write out the string
            CIcarus_BufferWrite(icarus, name_ptr as *const c_void, length as u64);
        }

        1
    }
}

// Get the current Game flavor.
pub fn CIcarus_GetFlavor(icarus: *const CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return -1;
        }
        (*icarus).m_flavor
    }
}

// int CIcarus::Save()
pub fn CIcarus_Save(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }

        // Allocate the temporary buffer.
        CIcarus_CreateBuffer(icarus);

        let icarus_ref = &*icarus;
        let game = IGameInterface_GetGame(icarus_ref.m_flavor);

        // Save out a ICARUS save block header with the ICARUS version
        let version = ICARUS_VERSION;
        if !game.is_null() {
            IGameInterface_WriteSaveData(game, b'I' as u32 | (b'C' as u32) << 8 | (b'A' as u32) << 16 | (b'R' as u32) << 24,
                &version as *const f64 as *const c_void, std::mem::size_of::<f64>() as u64);
        }

        // Save out the signals
        if CIcarus_SaveSignals(icarus) == 0 {
            CIcarus_DestroyBuffer(icarus);
            return 0;
        }

        // Save out the sequences
        if CIcarus_SaveSequences(icarus) == 0 {
            CIcarus_DestroyBuffer(icarus);
            return 0;
        }

        // Save out the sequencers
        if CIcarus_SaveSequencers(icarus) == 0 {
            CIcarus_DestroyBuffer(icarus);
            return 0;
        }

        let icarus_ref = &*icarus;
        // Write out the buffer with all our collected data.
        if !game.is_null() {
            IGameInterface_WriteSaveData(game, b'I' as u32 | (b'S' as u32) << 8 | (b'E' as u32) << 16 | (b'Q' as u32) << 24,
                icarus_ref.m_byBuffer as *const c_void, icarus_ref.m_ulBufferCurPos);
        }

        // De-allocate the temporary buffer.
        CIcarus_DestroyBuffer(icarus);

        1
    }
}

// int CIcarus::LoadSignals()
pub fn CIcarus_LoadSignals(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }
        let icarus_ref = &mut *icarus;

        let mut numSignals: c_int = 0;

        CIcarus_BufferRead(icarus, &mut numSignals as *mut c_int as *mut c_void, std::mem::size_of::<c_int>() as u64);

        for i in 0..numSignals {
            let mut buffer: [u8; 1024] = [0; 1024];
            let mut length: c_int = 0;

            // Get the size of the string
            CIcarus_BufferRead(icarus, &mut length as *mut c_int as *mut c_void, std::mem::size_of::<c_int>() as u64);

            // Get the string
            CIcarus_BufferRead(icarus, buffer.as_mut_ptr() as *mut c_void, length as u64);

            // Turn it on and add it to the system
            CIcarus_Signal(icarus, buffer.as_ptr() as *const c_char);
        }

        1
    }
}

// int CIcarus::LoadSequence()
pub fn CIcarus_LoadSequence(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }

        let sequence = CIcarus_GetSequence(icarus);

        // Load the sequence back in
        CSequence_Load(sequence, icarus);

        let icarus_ref = &mut *icarus;
        // If this sequence had a higher GUID than the current, save it
        if CSequence_GetID(sequence) > icarus_ref.m_GUID {
            icarus_ref.m_GUID = CSequence_GetID(sequence);
        }

        1
    }
}

// int CIcarus::LoadSequences()
pub fn CIcarus_LoadSequences(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }
        let icarus_ref = &mut *icarus;

        let mut numSequences: c_int = 0;

        // Get the number of sequences to read in
        CIcarus_BufferRead(icarus, &mut numSequences as *mut c_int as *mut c_void, std::mem::size_of::<c_int>() as u64);

        let idTable = libc::malloc((numSequences as usize) * std::mem::size_of::<c_int>()) as *mut c_int;

        if idTable.is_null() {
            return 0;
        }

        // Load the sequencer ID table
        CIcarus_BufferRead(icarus, idTable as *mut c_void, (std::mem::size_of::<c_int>() as c_int * numSequences) as u64);

        // First pass, allocate all container sequences and give them their proper IDs
        if CIcarus_AllocateSequences(icarus, numSequences, idTable) == 0 {
            libc::free(idTable as *mut c_void);
            return 0;
        }

        // Second pass, load all sequences
        for i in 0..numSequences {
            // Get the proper sequence for this load
            let sequence = CIcarus_GetSequence_ById(icarus, *idTable.add(i as usize));
            if sequence.is_null() {
                libc::free(idTable as *mut c_void);
                return 0;
            }

            // Load the sequence
            if CSequence_Load(sequence, icarus) == 0 {
                libc::free(idTable as *mut c_void);
                return 0;
            }
        }

        // Free the idTable
        libc::free(idTable as *mut c_void);

        1
    }
}

// int CIcarus::LoadSequencers()
pub fn CIcarus_LoadSequencers(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }
        let icarus_ref = &*icarus;

        let mut numSequencers: c_int = 0;
        let game = IGameInterface_GetGame(icarus_ref.m_flavor);

        // Get the number of sequencers to load
        CIcarus_BufferRead(icarus, &mut numSequencers as *mut c_int as *mut c_void, std::mem::size_of::<c_int>() as u64);

        // Load all sequencers
        for i in 0..numSequencers {
            // NOTENOTE: The ownerID will be replaced in the loading process
            let sequencerID = CIcarus_GetIcarusID(icarus, -1);
            let sequencer = CIcarus_FindSequencer(icarus, sequencerID);
            if sequencer.is_null() {
                return 0;
            }

            if CSequencer_Load(sequencer, icarus, game) == 0 {
                return 0;
            }
        }

        1
    }
}

// int CIcarus::Load()
pub fn CIcarus_Load(icarus: *mut CIcarus) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }

        CIcarus_CreateBuffer(icarus);

        let icarus_ref = &mut *icarus;
        let game = IGameInterface_GetGame(icarus_ref.m_flavor);

        // Clear out any old information
        CIcarus_Free(icarus);

        // Check to make sure we're at the ICARUS save block
        let mut version: f64 = 0.0;
        if !game.is_null() {
            IGameInterface_ReadSaveData(game, b'I' as u32 | (b'C' as u32) << 8 | (b'A' as u32) << 16 | (b'R' as u32) << 24,
                &mut version as *mut f64 as *mut c_void, std::mem::size_of::<f64>() as u64);
        }

        // Versions must match!
        if version != ICARUS_VERSION {
            CIcarus_DestroyBuffer(icarus);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "save game data contains outdated ICARUS version information!\n\0".as_ptr() as *const i8);
            }
            return 0;
        }

        // Read into the buffer all our data.
        if !game.is_null() {
            IGameInterface_ReadSaveData(game, b'I' as u32 | (b'S' as u32) << 8 | (b'E' as u32) << 16 | (b'Q' as u32) << 24,
                icarus_ref.m_byBuffer as *mut c_void, 0);
        }

        // Load all signals
        if CIcarus_LoadSignals(icarus) == 0 {
            CIcarus_DestroyBuffer(icarus);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "failed to load signals from save game!\n\0".as_ptr() as *const i8);
            }
            return 0;
        }

        // Load in all sequences
        if CIcarus_LoadSequences(icarus) == 0 {
            CIcarus_DestroyBuffer(icarus);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "failed to load sequences from save game!\n\0".as_ptr() as *const i8);
            }
            return 0;
        }

        // Load in all sequencers
        if CIcarus_LoadSequencers(icarus) == 0 {
            CIcarus_DestroyBuffer(icarus);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "failed to load sequencers from save game!\n\0".as_ptr() as *const i8);
            }
            return 0;
        }

        CIcarus_DestroyBuffer(icarus);

        1
    }
}

// int CIcarus::Update(int icarusID)
pub fn CIcarus_Update(icarus: *mut CIcarus, icarusID: c_int) -> c_int {
    unsafe {
        if icarus.is_null() {
            return -1;
        }

        let sequencer = CIcarus_FindSequencer(icarus, icarusID);
        if !sequencer.is_null() {
            let taskManager = CSequencer_GetTaskManager(sequencer);
            return CTaskManager_Update(taskManager, icarus);
        }
        -1
    }
}

// int CIcarus::IsRunning(int icarusID)
pub fn CIcarus_IsRunning(icarus: *const CIcarus, icarusID: c_int) -> c_int {
    unsafe {
        if icarus.is_null() {
            return 0;
        }

        let sequencer = CIcarus_FindSequencer(icarus, icarusID);
        if !sequencer.is_null() {
            let taskManager = CSequencer_GetTaskManager(sequencer);
            return CTaskManager_IsRunning(taskManager);
        }
        0
    }
}

// void CIcarus::Completed( int icarusID, int taskID )
pub fn CIcarus_Completed(icarus: *mut CIcarus, icarusID: c_int, taskID: c_int) {
    unsafe {
        if icarus.is_null() {
            return;
        }

        let sequencer = CIcarus_FindSequencer(icarus, icarusID);
        if !sequencer.is_null() {
            let taskManager = CSequencer_GetTaskManager(sequencer);
            CTaskManager_Completed(taskManager, taskID);
        }
    }
}

// void CIcarus::DestroyBuffer()
pub fn CIcarus_DestroyBuffer(icarus: *mut CIcarus) {
    unsafe {
        if icarus.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        if !icarus_ref.m_byBuffer.is_null() {
            let game = IGameInterface_GetGame(icarus_ref.m_flavor);
            if !game.is_null() {
                IGameInterface_Free(game, icarus_ref.m_byBuffer as *mut c_void);
            }
            icarus_ref.m_byBuffer = ptr::null_mut();
        }
    }
}

// void CIcarus::CreateBuffer()
pub fn CIcarus_CreateBuffer(icarus: *mut CIcarus) {
    unsafe {
        if icarus.is_null() {
            return;
        }

        CIcarus_DestroyBuffer(icarus);
        let icarus_ref = &mut *icarus;
        let game = IGameInterface_GetGame(icarus_ref.m_flavor);
        if !game.is_null() {
            icarus_ref.m_byBuffer = IGameInterface_Malloc(game, 100000) as *mut u8;
        }
        icarus_ref.m_ulBufferCurPos = 0;
    }
}

// void CIcarus::BufferWrite( void *pSrcData, unsigned long ulNumBytesToWrite )
pub fn CIcarus_BufferWrite(icarus: *mut CIcarus, pSrcData: *const c_void, ulNumBytesToWrite: u64) {
    unsafe {
        if icarus.is_null() || pSrcData.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        // Make sure we have enough space in the buffer to write to.
        if 100000u64 - icarus_ref.m_ulBufferCurPos < ulNumBytesToWrite {
            // Write out the buffer with all our collected data so far...
            let game = IGameInterface_GetGame(icarus_ref.m_flavor);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "BufferWrite: Out of buffer space, Flushing.\0".as_ptr() as *const i8);
                IGameInterface_WriteSaveData(game, b'I' as u32 | (b'S' as u32) << 8 | (b'E' as u32) << 16 | (b'Q' as u32) << 24,
                    icarus_ref.m_byBuffer as *const c_void, icarus_ref.m_ulBufferCurPos);
            }
            icarus_ref.m_ulBufferCurPos = 0; // reset buffer
        }

        assert!(100000u64 - icarus_ref.m_ulBufferCurPos >= ulNumBytesToWrite);
        {
            libc::memcpy(icarus_ref.m_byBuffer.add(icarus_ref.m_ulBufferCurPos as usize) as *mut c_void, pSrcData, ulNumBytesToWrite as usize);
            icarus_ref.m_ulBufferCurPos += ulNumBytesToWrite;
        }
    }
}

// void CIcarus::BufferRead( void *pDstBuff, unsigned long ulNumBytesToRead )
pub fn CIcarus_BufferRead(icarus: *mut CIcarus, pDstBuff: *mut c_void, ulNumBytesToRead: u64) {
    unsafe {
        if icarus.is_null() || pDstBuff.is_null() {
            return;
        }
        let icarus_ref = &mut *icarus;

        // If we can read this data...
        if icarus_ref.m_ulBytesRead + ulNumBytesToRead > 100000u64 {
            // We've tried to read past the buffer...
            let game = IGameInterface_GetGame(icarus_ref.m_flavor);
            if !game.is_null() {
                IGameInterface_DebugPrint(game, 2, "BufferRead: Buffer underflow, Looking for new block.\0".as_ptr() as *const i8);
                // Read in the next block.
                IGameInterface_ReadSaveData(game, b'I' as u32 | (b'S' as u32) << 8 | (b'E' as u32) << 16 | (b'Q' as u32) << 24,
                    icarus_ref.m_byBuffer as *mut c_void, 0);
            }
            icarus_ref.m_ulBytesRead = 0; // reset buffer
        }

        assert!(icarus_ref.m_ulBytesRead + ulNumBytesToRead <= 100000u64);
        {
            libc::memcpy(pDstBuff, icarus_ref.m_byBuffer.add(icarus_ref.m_ulBytesRead as usize) as *const c_void, ulNumBytesToRead as usize);
            icarus_ref.m_ulBytesRead += ulNumBytesToRead;
        }
    }
}

// LOCAL STUBS: External function declarations
extern "C" {
    fn CSequencer_Create() -> *mut CSequencer;
    fn CSequencer_Free(sequencer: *mut CSequencer, icarus: *mut CIcarus);
    fn CSequencer_Init(sequencer: *mut CSequencer, gameID: c_int, taskManager: *mut CTaskManager);
    fn CSequencer_GetID(sequencer: *const CSequencer) -> c_int;
    fn CSequencer_GetTaskManager(sequencer: *mut CSequencer) -> *mut CTaskManager;
    fn CSequencer_Recall(sequencer: *mut CSequencer, icarus: *mut CIcarus);
    fn CSequencer_Run(sequencer: *mut CSequencer, buffer: *mut c_char, length: i64, icarus: *mut CIcarus) -> c_int;
    fn CSequencer_Save(sequencer: *mut CSequencer);
    fn CSequencer_Load(sequencer: *mut CSequencer, icarus: *mut CIcarus, game: *mut IGameInterface) -> c_int;

    fn CTaskManager_Create() -> *mut CTaskManager;
    fn CTaskManager_Free(taskManager: *mut CTaskManager);
    fn CTaskManager_Init(taskManager: *mut CTaskManager, sequencer: *mut CSequencer);
    fn CTaskManager_IsResident(taskManager: *mut CTaskManager) -> c_int;
    fn CTaskManager_IsRunning(taskManager: *mut CTaskManager) -> c_int;
    fn CTaskManager_Update(taskManager: *mut CTaskManager, icarus: *mut CIcarus) -> c_int;
    fn CTaskManager_Completed(taskManager: *mut CTaskManager, taskID: c_int);

    fn CSequence_Create() -> *mut CSequence;
    fn CSequence_Delete(sequence: *mut CSequence, icarus: *mut CIcarus);
    fn CSequence_SetID(sequence: *mut CSequence, id: c_int);
    fn CSequence_GetID(sequence: *const CSequence) -> c_int;
    fn CSequence_Save(sequence: *mut CSequence);
    fn CSequence_Load(sequence: *mut CSequence, icarus: *mut CIcarus) -> c_int;

    fn CBlockStream_Open(stream: *mut CBlockStream, buffer: *mut c_char, length: i64) -> c_int;
    fn CBlockStream_BlockAvailable(stream: *const CBlockStream) -> c_int;
    fn CBlockStream_ReadBlock(stream: *mut CBlockStream, block: *mut CBlock, icarus: *mut c_void) -> c_int;
    fn CBlockStream_Free(stream: *mut CBlockStream);

    fn CBlock_Free(block: *mut CBlock, icarus: *mut c_void);
    fn CBlock_GetBlockID(block: *const CBlock) -> c_int;
    fn CBlock_GetMemberData(block: *const CBlock, index: c_int) -> *mut c_void;
    fn CBlock_GetMember(block: *const CBlock, index: c_int) -> *mut CBlockMember;

    fn CBlockMember_GetID(member: *const CBlockMember) -> c_int;

    fn IGameInterface_GetGame(flavor: c_int) -> *mut IGameInterface;
    fn IGameInterface_Free(game: *mut IGameInterface, ptr: *mut c_void);
    fn IGameInterface_Malloc(game: *mut IGameInterface, size: usize) -> *mut c_void;
    fn IGameInterface_DebugPrint(game: *mut IGameInterface, level: c_int, fmt: *const c_char, ...) -> c_int;
    fn IGameInterface_WriteSaveData(game: *mut IGameInterface, blockid: u32, data: *const c_void, length: u64);
    fn IGameInterface_ReadSaveData(game: *mut IGameInterface, blockid: u32, data: *mut c_void, length: u64);
    fn IGameInterface_PrecacheRoff(game: *mut IGameInterface, filename: *const c_char);
    fn IGameInterface_PrecacheScript(game: *mut IGameInterface, filename: *const c_char);
    fn IGameInterface_PrecacheSound(game: *mut IGameInterface, filename: *const c_char);
    fn IGameInterface_PrecacheFromSet(game: *mut IGameInterface, key: *const c_char, value: *const c_char);
}

// Helper function for case-insensitive string comparison
fn strcmp_safe(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        if s1.is_null() || s2.is_null() {
            return if s1 == s2 { 0 } else { -1 };
        }
        libc::strcasecmp(s1, s2)
    }
}

// Local stub: stricmp for case-insensitive comparison
fn stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    strcmp_safe(s1, s2)
}
