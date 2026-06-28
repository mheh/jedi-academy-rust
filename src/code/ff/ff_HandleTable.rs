//! Mechanical port of `code/ff/ff_HandleTable.cpp`.
//!
//! Porting note: The original file was wrapped in `#ifdef _IMMERSION`, but this
//! code is translated unconditionally since _IMMERSION is not yet a Cargo feature.

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use core::ffi::{c_char, c_int};
use std::collections::BTreeMap;

// LOCAL STUB: qboolean type from engine
pub type qboolean = c_int;

// LOCAL STUB: ffHandle_t type from ff_public.h
pub type ffHandle_t = c_int;

// LOCAL STUB: TNameTable type from ff_utils.h (vector<string>)
pub type TNameTable = Vec<String>;

// LOCAL STUB: Constants from ff_public.h
pub const FF_HANDLE_NULL: c_int = 0;

// LOCAL STUB: ChannelCompound - forward declaration stub
// Real definition is in ff_ChannelCompound_h.rs
// This stub provides the minimal interface needed for FFHandleTable::Convert methods
pub struct ChannelCompound;

impl ChannelCompound {
    pub fn new() -> Self {
        ChannelCompound
    }

    pub fn GetSet_size(&self) -> usize {
        0 // Stub
    }

    pub fn IsEmpty(&self) -> qboolean {
        0 // Stub
    }

    pub fn GetName(&self) -> *const c_char {
        core::ptr::null()
    }
}

impl Clone for ChannelCompound {
    fn clone(&self) -> Self {
        ChannelCompound
    }
}

impl PartialEq for ChannelCompound {
    fn eq(&self, _other: &Self) -> bool {
        false // Stub
    }
}

////-----------------------
/// FFHandleTable
////-----------------------
/// This table houses the master list of initialized effects. Indices
/// into this table are handles used by external modules. This way
/// effects may be reinitialized on other devices, removed entirely,
/// and perused informatively at any time without invalidating pointers.
///
#[repr(C)]
pub struct FFHandleTable {
    // typedef vector<ChannelCompound> Vector;
    mVector: Vec<ChannelCompound>,
    // typedef map<int, string> RegFail;
    mRegFail: BTreeMap<ffHandle_t, String>,
}

impl FFHandleTable {
    pub fn new() -> Self {
        FFHandleTable {
            mVector: Vec::new(),
            mRegFail: BTreeMap::new(),
        }
    }

    pub fn Init(&mut self) {
        let handle_null = ChannelCompound::new();
        self.mVector.push_back(handle_null);
    }

    // Empties handle table except for FF_HANDLE_NULL
    pub fn clear(&mut self) {
        self.mVector.clear();
        self.mRegFail.clear();
    }

    pub fn size(&self) -> usize {
        self.mVector.len()
    }

    pub fn index(&mut self, ff: ffHandle_t) -> &mut ChannelCompound {
        // InRange<int>( ff, 0, mVector.size() - 1, FF_HANDLE_NULL )
        let in_range_index = if ff < 0 || ff > (self.mVector.len() as ffHandle_t - 1) {
            FF_HANDLE_NULL as usize
        } else {
            ff as usize
        };
        &mut self.mVector[in_range_index]
    }

    ////----------------------
    /// FFHandleTable::Convert
    //--------------------------
    //
    //
    pub fn Convert_with_name(
        &mut self,
        compound: &ChannelCompound,
        name: *const c_char,
        create: qboolean,
    ) -> ffHandle_t {
        let mut ff = FF_HANDLE_NULL;

        //	Reserve a handle for effects that failed to create.
        //	Rerouting channels to other devices may cause an effect to become lost.
        //	This assumes that FF_Register is always called with legitimate effect names.
        //	See CMD_FF_Play on how to handle possibly-bogus user input.
        //	(It does not call this function)
        if compound.GetSet_size() > 0 {
            ff = self.Convert(compound);
        } else {
            // Mechanical port of C++ for loop:
            // for
            // (	FFHandleTable::RegFail::iterator itRegFail = mRegFail.begin()
            // ;	itRegFail != mRegFail.end()
            // &&	(*itRegFail).second != name
            // ;	itRegFail++
            // );
            // ff = (itRegFail != mRegFail.end() ? (*itRegFail).first : FF_HANDLE_NULL);

            let mut found_handle = FF_HANDLE_NULL;
            if !name.is_null() {
                let name_str = unsafe {
                    std::ffi::CStr::from_ptr(name)
                        .to_string_lossy()
                        .to_string()
                };
                for (handle, stored_name) in self.mRegFail.iter() {
                    if *stored_name == name_str {
                        found_handle = *handle;
                        break;
                    }
                }
            }
            ff = found_handle;
        }

        if ff == FF_HANDLE_NULL {
            self.mVector.push_back(compound.clone());
            ff = (self.mVector.len() - 1) as ffHandle_t;

            // Remember effect name for future 'ff_restart' calls.
            if create != 0 && compound.IsEmpty() != 0 {
                if !name.is_null() {
                    let name_str = unsafe {
                        std::ffi::CStr::from_ptr(name)
                            .to_string_lossy()
                            .to_string()
                    };
                    self.mRegFail.insert(ff, name_str);
                }
            }
        }

        ff
    }

    ////----------------------
    /// FFHandleTable::Convert
    //--------------------------
    //	Looks for 'compound' in the table.
    //
    //	Assumes:
    //	*	'compound' is non-empty
    //
    //	Returns:
    //		ffHandle_t
    //
    pub fn Convert(&self, compound: &ChannelCompound) -> ffHandle_t {
        // for (int i = 1
        // ;	i < mVector.size()
        // &&	mVector[ i ] != compound
        // ;	i++);
        //
        // return (i < mVector.size() ? i : FF_HANDLE_NULL);

        let mut i = 1;
        while i < self.mVector.len() {
            if self.mVector[i] == *compound {
                return i as ffHandle_t;
            }
            i += 1;
        }
        FF_HANDLE_NULL
    }

    ////-----------------------------
    /// FFHandleTable::GetFailedNames
    //---------------------------------
    //
    //
    pub fn GetFailedNames(&self, name_table: &mut TNameTable) -> qboolean {
        // for (RegFail::iterator itRegFail = mRegFail.begin()
        // ;	itRegFail != mRegFail.end()
        // ;	itRegFail++)
        // {
        //     NameTable[ (*itRegFail).first ] = (*itRegFail).second;
        // }
        //
        // return qboolean( mRegFail.size() != 0 );

        for (handle, name) in self.mRegFail.iter() {
            // Resize name_table if needed to accommodate the index
            if (*handle as usize) >= name_table.len() {
                name_table.resize(*handle as usize + 1, String::new());
            }
            name_table[*handle as usize] = name.clone();
        }

        if self.mRegFail.len() != 0 { 1 } else { 0 }
    }

    ////--------------------------
    /// FFHandleTable::GetChannels
    //------------------------------
    //
    //
    pub fn GetChannels(&self, channel: &mut Vec<c_int>) -> qboolean {
        //ASSERT( channel.size() >= mVector.size() );

        // for (int i = 1
        // ;	i < mVector.size()
        // ;	i++)
        // {
        //     channel[ i ] = mVector[ i ].GetChannel();
        // }
        //
        // return qtrue;

        for i in 1..self.mVector.len() {
            if i >= channel.len() {
                channel.resize(i + 1, 0);
            }
            // Stub: GetChannel() implementation will come from ChannelCompound
            channel[i] = 0; // Placeholder, will be replaced with actual GetChannel()
        }

        1 // return qtrue
    }

    pub fn GetName(&self, ff: ffHandle_t) -> *const c_char {
        let mut result: *const c_char = core::ptr::null();

        if ff >= 0 && (ff as usize) < self.mVector.len() {
            if self.mVector[ff as usize].IsEmpty() == 0 {
                result = self.mVector[ff as usize].GetName();
            } else {
                if let Some(name) = self.mRegFail.get(&ff) {
                    // SAFETY: This is safe because we're converting a &String to a *const c_char
                    // The string must remain valid as long as the pointer is used
                    result = name.as_ptr() as *const c_char;
                }
            }
        }

        result
    }
}

