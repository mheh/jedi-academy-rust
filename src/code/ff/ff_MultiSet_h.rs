#![allow(non_snake_case)]

use core::ffi::c_char;

// LOCAL STUB: qboolean type from engine
pub type qboolean = core::ffi::c_int;

// LOCAL STUB: TNameTable type from ff_utils.h
pub type TNameTable = Vec<String>;

// Forward declarations for types from other modules
// FFSet from ff_ffset.h
pub struct FFSet {
    _private: (),
}

// FFConfigParser - opaque type from ff_ConfigParser.h
#[repr(C)]
pub struct FFConfigParser {
    _opaque: [u8; 0],
}

// CImmDevices from IFC - opaque type
#[repr(C)]
pub struct CImmDevices {
    _opaque: [u8; 0],
}

//===[FFMultiSet]=====================================================/////////////
//
//	A collection class of FFSet objects. These functions generally
//	iterate over the entire set of FFSets, performing the same operation
//	on all contained FFSets.
//
//====================================================================/////////////

#[repr(C)]
pub struct FFMultiSet {
    mConfig: *mut FFConfigParser,
    // typedef vector<FFSet*> Set;
    // Set mSet;
    mSet: Vec<*mut FFSet>,
    mDevices: *mut CImmDevices,
}

impl FFMultiSet {
    // qboolean Init( FFConfigParser &config );
    pub fn Init(&mut self, config: &mut FFConfigParser) -> qboolean {
        // Stub: implementation will be in .cpp file
        0
    }

    // qboolean GetRegisteredNames( TNameTable &NameTable );
    pub fn GetRegisteredNames(&mut self, NameTable: &mut TNameTable) -> qboolean {
        // Stub: implementation will be in .cpp file
        0
    }

    // qboolean StopAll();
    pub fn StopAll(&mut self) -> qboolean {
        // Stub: implementation will be in .cpp file
        0
    }

    // void clear();
    pub fn clear(&mut self) {
        self.mSet.clear();
    }
}

//
//	Optional
//
// #ifdef FF_ACCESSOR
#[cfg(feature = "FF_ACCESSOR")]
impl FFMultiSet {
    // Set& GetSets() { return mSet; }
    pub fn GetSets(&mut self) -> &mut Vec<*mut FFSet> {
        &mut self.mSet
    }

    // CImmDevices* GetDevices() { return mDevices; }
    pub fn GetDevices(&self) -> *mut CImmDevices {
        self.mDevices
    }
}
// #endif

// #ifdef FF_CONSOLECOMMAND
#[cfg(feature = "FF_CONSOLECOMMAND")]
impl FFMultiSet {
    // void Display( TNameTable &Unprocessed, TNameTable &Processed );
    pub fn Display(&mut self, Unprocessed: &mut TNameTable, Processed: &mut TNameTable) {
        // Stub: implementation will be in .cpp file
    }

    // static void GetDisplayTokens( TNameTable &Tokens );
    pub fn GetDisplayTokens(Tokens: &mut TNameTable) {
        // Stub: implementation will be in .cpp file
    }
}
// #endif
